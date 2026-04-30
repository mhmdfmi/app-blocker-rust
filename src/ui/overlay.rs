/// Overlay fullscreen Win32 menggunakan GDI.
/// Topmost, tidak bisa关闭, blokir Alt+F4 dan Escape.
/// Fix: WM_TIMER ID 99 (close setelah unlock), failsafe timeout, semua imports lengkap.
/// Update: Menambahkan heartbeat ke watchdog untuk监控 UI overlay.
use crate::core::events::AppEvent;
use crate::core::events::ComponentId;
use crate::core::watchdog::send_watchdog_heartbeat;
use crate::security::auth::{AuthManager, AuthStatus};
use crate::ui::components::{theme, CardLayout, DisplayData};
#[cfg(target_os = "windows")]
use crate::ui::window::{get_screen_dimensions, to_wide};
use crate::utils::error::{AppError, AppResult};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};
use uuid::Uuid;
use windows::Win32::Foundation::HWND;
// ID Timer
const TIMER_TOPMOST: usize = 1; // Re-topmost setiap 500ms
const TIMER_CLOCK: usize = 2; // Update jam setiap detik
const TIMER_CLOSE_OVERLAY: usize = 99; // Tutup overlay 800ms setelah unlock
const TIMER_FAILSAFE: usize = 10; // Failsafe timeout (30 menit)

/// State yang disimpan per window instance
struct OverlayState {
    auth_manager: Arc<Mutex<AuthManager>>,
    event_tx: Sender<AppEvent>,
    display: DisplayData,
    trace_id: Uuid,
    status_msg: String,
    status_is_error: bool,
    failsafe_minutes: u64,
    should_close: bool,
}

/// Jalankan overlay fullscreen (blocking - jalankan di thread UI)
#[cfg(target_os = "windows")]
pub fn run_overlay(
    display: DisplayData,
    auth_manager: Arc<Mutex<AuthManager>>,
    event_tx: Sender<AppEvent>,
    trace_id: Uuid,
    failsafe_minutes: u64,
) -> AppResult<()> {
    #[cfg(target_os = "windows")]
    {
        run_overlay_win32(display, auth_manager, event_tx, trace_id, failsafe_minutes)
    }
    #[cfg(not(target_os = "windows"))]
    {
        run_overlay_stub(display, auth_manager, event_tx, trace_id)
    }
}

#[cfg(target_os = "windows")]
fn run_overlay_win32(
    display: DisplayData,
    auth_manager: Arc<Mutex<AuthManager>>,
    event_tx: Sender<AppEvent>,
    trace_id: Uuid,
    failsafe_minutes: u64,
) -> AppResult<()> {
    use windows::{
        core::PCWSTR,
        Win32::{
            Foundation::COLORREF,
            Graphics::Gdi::{CreateSolidBrush, DeleteObject},
            System::LibraryLoader::GetModuleHandleW,
            UI::WindowsAndMessaging::{
                CreateWindowExW, DispatchMessageW, GetMessageW, RegisterClassExW, SetTimer,
                SetWindowPos, ShowWindow, TranslateMessage, UnregisterClassW, CS_HREDRAW,
                CS_VREDRAW, HWND_TOPMOST, MSG, SWP_SHOWWINDOW, SW_SHOW, WNDCLASSEXW, WS_EX_TOPMOST,
                WS_POPUP, WS_VISIBLE,
            },
        },
    };

    // info!(%trace_id, pid = display.pid, "Memulai overlay Win32");

    let (screen_w, screen_h) = get_screen_dimensions();
    let class_name_w = to_wide("AppBlockerOverlayClass_v2");
    let title_w = to_wide("Peringatan Keamanan - App Blocker");

    let state = Box::new(OverlayState {
        auth_manager,
        event_tx,
        display: display.clone(),
        trace_id,
        status_msg: String::new(),
        status_is_error: false,
        failsafe_minutes,
        should_close: false,
    });
    let state_ptr = Box::into_raw(state) as isize;

    let module = unsafe {
        GetModuleHandleW(None).map_err(|e| AppError::Win32(format!("GetModuleHandle: {e}")))?
    };

    // Register window class
    let bg_brush = unsafe {
        CreateSolidBrush(COLORREF(theme::to_colorref(
            theme::BG_MAIN.0,
            theme::BG_MAIN.1,
            theme::BG_MAIN.2,
        )))
    };

    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(overlay_wnd_proc),
        hInstance: module.into(),
        lpszClassName: PCWSTR(class_name_w.as_ptr()),
        hbrBackground: bg_brush,
        ..Default::default()
    };
    unsafe { RegisterClassExW(&wc) };

    // Buat window fullscreen
    // FIX: Hapus WS_EX_LAYERED karena menyebabkan overlay transparan/invisible
    // Jika ingin transparansi, gunakan SetLayeredWindowAttributes setelah CreateWindowEx
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_TOPMOST,
            PCWSTR(class_name_w.as_ptr()),
            PCWSTR(title_w.as_ptr()),
            WS_POPUP | WS_VISIBLE,
            0,
            0,
            screen_w,
            screen_h,
            None,
            None,
            module,
            Some(state_ptr as *const _),
        )
        // .map_err(|e| AppError::Win32(format!("CreateWindowEx: {e}")))?
    };

    unsafe {
        ShowWindow(hwnd, SW_SHOW);
        SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, screen_w, screen_h, SWP_SHOWWINDOW)
            .map_err(|e| AppError::Win32(format!("SetWindowPos: {e}")))?;

        // Timer: re-topmost setiap 500ms
        SetTimer(hwnd, TIMER_TOPMOST, 500, None);
        // Timer: update jam setiap detik
        SetTimer(hwnd, TIMER_CLOCK, 1000, None);
        // Timer: failsafe (tutup otomatis setelah N menit)
        let failsafe_ms = (failsafe_minutes * 60 * 1000) as u32;
        SetTimer(hwnd, TIMER_FAILSAFE, failsafe_ms, None);
    }

    info!(
        "Overlay ditampilkan (failsafe: {}m), menjalankan message loop",
        failsafe_minutes
    );

    // Message loop - kirim heartbeat setiap 1 detik ke watchdog
    let mut msg = MSG::default();
    let mut last_hb = std::time::Instant::now();
    loop {
        let r = unsafe { GetMessageW(&mut msg, None, 0, 0) };
        if r.0 <= 0 {
            break;
        }
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        // Kirim heartbeat setiap 1 detik
        if last_hb.elapsed().as_millis() >= 1000 {
            send_watchdog_heartbeat(ComponentId::UiOverlay);
            last_hb = std::time::Instant::now();
        }
    }

    // Bersihkan class dan brush
    unsafe {
        let _ = UnregisterClassW(PCWSTR(class_name_w.as_ptr()), module);
        let _ = DeleteObject(bg_brush);
    }

    info!(%trace_id, "Overlay Win32 ditutup");
    Ok(())
}

/// WndProc utama overlay
#[cfg(target_os = "windows")]
unsafe extern "system" fn overlay_wnd_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg_id: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use crate::ui::components::ctrl_id;
    use crate::ui::window::to_wide;
    use windows::Win32::{
        Foundation::LRESULT,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, GetWindowLongPtrW, KillTimer, PostQuitMessage,
            SetWindowLongPtrW, SetWindowPos, BN_CLICKED, ES_AUTOHSCROLL, ES_PASSWORD,
            GWLP_USERDATA, HMENU, HWND_TOPMOST, SC_CLOSE, SWP_NOMOVE, SWP_NOSIZE, WM_CLOSE,
            WM_COMMAND, WM_CREATE, WM_DESTROY, WM_KEYDOWN, WM_PAINT, WM_SYSCOMMAND, WM_SYSKEYDOWN,
            WM_TIMER, WS_BORDER, WS_CHILD, WS_EX_CLIENTEDGE, WS_VISIBLE,
        },
    };

    let wm_create = WM_CREATE;
    let wm_paint = WM_PAINT;
    let wm_timer = WM_TIMER;
    let wm_command = WM_COMMAND;
    let wm_close = WM_CLOSE;
    let wm_destroy = WM_DESTROY;
    let wm_syscommand = WM_SYSCOMMAND;
    let wm_syskeydown = WM_SYSKEYDOWN;
    let wm_keydown = WM_KEYDOWN;

    match msg_id {
        // Init: simpan pointer state
        m if m == wm_create => {
            use windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE;

            let cs = &*(lparam.0 as *const windows::Win32::UI::WindowsAndMessaging::CREATESTRUCTW);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, cs.lpCreateParams as isize);

            let _state = &*(cs.lpCreateParams as *mut OverlayState);
            let (sw, sh) = get_screen_dimensions();
            let card = CardLayout::centered(sw, sh);

            let module =
                windows::Win32::System::LibraryLoader::GetModuleHandleW(None).unwrap_or_default();

            let style_bits: u32 = (ES_PASSWORD as u32)
                | (ES_AUTOHSCROLL as u32)
                | WS_CHILD.0
                | WS_VISIBLE.0
                | WS_BORDER.0;
            // Jika tipe internal adalah i32/u32, .0 sudah sesuai; bungkus kembali:
            let style = WINDOW_STYLE(style_bits);

            // Edit control password
            let edit_class = to_wide("EDIT");
            let empty_txt = to_wide("");
            CreateWindowExW(
                WS_EX_CLIENTEDGE,
                windows::core::PCWSTR(edit_class.as_ptr()),
                windows::core::PCWSTR(empty_txt.as_ptr()),
                style,
                card.x + 24,
                card.y + 290,
                card.w - 48,
                40,
                hwnd,
                HMENU(ctrl_id::ID_INPUT_PASSWORD as isize),
                module,
                None,
            );

            // Tombol Buka Kunci
            let btn_class = to_wide("BUTTON");
            let btn_txt = to_wide("  Buka Kunci");
            CreateWindowExW(
                windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE::default(),
                windows::core::PCWSTR(btn_class.as_ptr()),
                windows::core::PCWSTR(btn_txt.as_ptr()),
                WS_CHILD | WS_VISIBLE,
                card.x + 24,
                card.y + 348,
                card.w - 48,
                44,
                hwnd,
                HMENU(ctrl_id::ID_BTN_SUBMIT as isize),
                module,
                None,
            );

            LRESULT(0)
        }

        // Blokir Alt+F4 dan SC_CLOSE
        m if m == wm_syscommand => {
            if wparam.0 == SC_CLOSE as usize {
                return LRESULT(0);
            }
            DefWindowProcW(hwnd, msg_id, wparam, lparam)
        }

        // Jangan tutup kecuali sudah di-unlock
        m if m == wm_close || m == wm_destroy => {
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
            if state_ptr != 0 {
                let state = &*(state_ptr as *mut OverlayState);
                if state.should_close {
                    // Benar-benar tutup
                    PostQuitMessage(0);
                }
            }
            LRESULT(0)
        }

        // Blokir Escape dan F4
        m if m == wm_syskeydown || m == wm_keydown => {
            match wparam.0 as u32 {
                0x1B | 0x73 => LRESULT(0), // Escape, F4
                0x0D => {
                    // Enter = submit
                    handle_submit(hwnd);
                    LRESULT(0)
                }
                _ => DefWindowProcW(hwnd, msg_id, wparam, lparam),
            }
        }

        // Timer events
        m if m == wm_timer => {
            match wparam.0 {
                TIMER_TOPMOST => {
                    let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
                }
                TIMER_CLOCK => {
                    // Repaint untuk update jam
                    let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
                }
                TIMER_CLOSE_OVERLAY => {
                    // ✅ FIX: Timer 99 - benar-benar tutup overlay setelah unlock
                    KillTimer(hwnd, TIMER_CLOSE_OVERLAY).ok();
                    KillTimer(hwnd, TIMER_TOPMOST).ok();
                    KillTimer(hwnd, TIMER_CLOCK).ok();
                    KillTimer(hwnd, TIMER_FAILSAFE).ok();

                    // Set flag agar WM_DESTROY tidak diabaikan
                    let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
                    if state_ptr != 0 {
                        let state = &mut *(state_ptr as *mut OverlayState);
                        state.should_close = true;
                    }

                    PostQuitMessage(0);
                }
                TIMER_FAILSAFE => {
                    // Failsafe timeout - tutup overlay otomatis
                    KillTimer(hwnd, TIMER_FAILSAFE).ok();
                    warn!("Failsafe timeout tercapai - menutup overlay otomatis");

                    let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
                    if state_ptr != 0 {
                        let state = &mut *(state_ptr as *mut OverlayState);
                        state.should_close = true;
                        // Kirim event untuk reset state engine
                        let _ = state.event_tx.send(AppEvent::UnlockSuccess {
                            trace_id: state.trace_id,
                            username: "FAILSAFE_AUTO_UNLOCK".to_string(),
                            unlocked_at: crate::utils::time::now_utc(),
                        });
                    }

                    PostQuitMessage(0);
                }
                _ => {}
            }
            LRESULT(0)
        }

        // Klik tombol submit
        m if m == wm_command => {
            let ctrl = (wparam.0 & 0xFFFF) as i32;
            let notif = (wparam.0 >> 16) as u32;
            if ctrl == crate::ui::components::ctrl_id::ID_BTN_SUBMIT && notif == BN_CLICKED {
                handle_submit(hwnd);
            }
            LRESULT(0)
        }

        // Paint: render seluruh overlay
        m if m == wm_paint => {
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
            if state_ptr == 0 {
                return DefWindowProcW(hwnd, msg_id, wparam, lparam);
            }
            let state = &*(state_ptr as *mut OverlayState);
            render_overlay(hwnd, state);
            LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg_id, wparam, lparam),
    }
}

/// Render seluruh UI overlay dengan GDI
#[cfg(target_os = "windows")]
unsafe fn render_overlay(hwnd: windows::Win32::Foundation::HWND, state: &OverlayState) {
    use crate::ui::components::theme;
    use windows::Win32::{
        Foundation::COLORREF,
        Graphics::Gdi::{
            BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, EndPaint, FillRect,
            SelectObject, SetBkMode, SetTextColor, TextOutW, CLEARTYPE_QUALITY,
            CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, FF_SWISS, FW_BOLD, FW_NORMAL, OUT_DEFAULT_PRECIS,
            PAINTSTRUCT, TRANSPARENT,
        },
    };

    let (sw, sh) = get_screen_dimensions();
    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(hwnd, &mut ps);

    // ── Background gelap
    let bg_brush = CreateSolidBrush(COLORREF(theme::to_colorref(
        theme::BG_MAIN.0,
        theme::BG_MAIN.1,
        theme::BG_MAIN.2,
    )));
    let full = windows::Win32::Foundation::RECT {
        left: 0,
        top: 0,
        right: sw,
        bottom: sh,
    };
    FillRect(hdc, &full, bg_brush);
    let _ = DeleteObject(bg_brush);

    // ── Kartu tengah
    let card = CardLayout::centered(sw, sh);
    let card_brush = CreateSolidBrush(COLORREF(theme::to_colorref(
        theme::BG_CARD.0,
        theme::BG_CARD.1,
        theme::BG_CARD.2,
    )));
    let card_rect = windows::Win32::Foundation::RECT {
        left: card.x,
        top: card.y,
        right: card.x + card.w,
        bottom: card.y + card.h,
    };
    FillRect(hdc, &card_rect, card_brush);
    let _ = DeleteObject(card_brush);

    // ── Helper: draw_text closure
    let draw_text = |text: &str, x: i32, y: i32, size: i32, bold: bool, r: u8, g: u8, b: u8| {
        let face = to_wide("Segoe UI");
        let weight = if bold {
            FW_BOLD.0 as i32
        } else {
            FW_NORMAL.0 as i32
        };
        let font = CreateFontW(
            size,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET.0 as u32,
            OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            CLEARTYPE_QUALITY.0 as u32,
            FF_SWISS.0 as u32,
            windows::core::PCWSTR(face.as_ptr()),
        );
        let old = SelectObject(hdc, font);
        SetTextColor(hdc, COLORREF(theme::to_colorref(r, g, b)));
        SetBkMode(hdc, TRANSPARENT);
        let w = to_wide(text);
        TextOutW(hdc, x, y, &w[..w.len() - 1]);
        SelectObject(hdc, old);
        let _ = DeleteObject(font);
    };

    // ── Garis merah atas kartu
    let red_brush = CreateSolidBrush(COLORREF(theme::to_colorref(
        theme::RED_DANGER.0,
        theme::RED_DANGER.1,
        theme::RED_DANGER.2,
    )));
    FillRect(
        hdc,
        &windows::Win32::Foundation::RECT {
            left: card.x,
            top: card.y,
            right: card.x + card.w,
            bottom: card.y + 6,
        },
        red_brush,
    );
    let _ = DeleteObject(red_brush);

    // ── Konten teks
    let (tr, tg, tb) = theme::TEXT_WHITE;
    let (mr, mg, mb) = theme::TEXT_MUTED;
    let (yr, yg, yb) = theme::YELLOW_WARN;
    let (rr, rg, rb) = theme::RED_DANGER;

    draw_text(
        "  PERINGATAN KEAMANAN",
        card.x + 24,
        card.y + 18,
        22,
        true,
        rr,
        rg,
        rb,
    );

    draw_text(
        "Aplikasi terlarang telah terdeteksi dan dihentikan oleh sistem.",
        card.x + 24,
        card.y + 60,
        13,
        false,
        tr,
        tg,
        tb,
    );
    draw_text(
        "Masukkan kata sandi administrator untuk melanjutkan.",
        card.x + 24,
        card.y + 82,
        12,
        false,
        mr,
        mg,
        mb,
    );

    // Divider
    let div_brush = CreateSolidBrush(COLORREF(theme::to_colorref(
        theme::BORDER_CARD.0,
        theme::BORDER_CARD.1,
        theme::BORDER_CARD.2,
    )));
    FillRect(
        hdc,
        &windows::Win32::Foundation::RECT {
            left: card.x + 24,
            top: card.y + 112,
            right: card.x + card.w - 24,
            bottom: card.y + 114,
        },
        div_brush,
    );
    let _ = DeleteObject(div_brush);

    // Info proses
    draw_text(
        &format!(
            "Proses   : {}  (PID: {})",
            state.display.process_name, state.display.pid
        ),
        card.x + 24,
        card.y + 126,
        12,
        false,
        yr,
        yg,
        yb,
    );
    draw_text(
        &format!("Pengguna : {}", state.display.username),
        card.x + 24,
        card.y + 148,
        12,
        false,
        mr,
        mg,
        mb,
    );
    draw_text(
        &format!("Komputer : {}", state.display.computer_name),
        card.x + 24,
        card.y + 166,
        12,
        false,
        mr,
        mg,
        mb,
    );
    draw_text(
        &format!(
            "Waktu    : {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        ),
        card.x + 24,
        card.y + 184,
        12,
        false,
        mr,
        mg,
        mb,
    );

    // Percobaan
    let att_text = format!(
        "Percobaan: {}/{}",
        state.display.attempts, state.display.max_attempts
    );
    let (ar, ag, ab) = if state.display.attempts >= state.display.max_attempts / 2 {
        theme::RED_DANGER
    } else {
        theme::TEXT_MUTED
    };
    draw_text(&att_text, card.x + 24, card.y + 222, 12, false, ar, ag, ab);

    // Label password
    draw_text(
        "Kata Sandi Administrator",
        card.x + 24,
        card.y + 270,
        12,
        true,
        tr,
        tg,
        tb,
    );

    // Status pesan
    if !state.status_msg.is_empty() {
        let (sr, sg, sb) = if state.status_is_error {
            theme::RED_DANGER
        } else {
            theme::GREEN_SUCCESS
        };
        draw_text(
            &state.status_msg,
            card.x + 24,
            card.y + 408,
            13,
            true,
            sr,
            sg,
            sb,
        );
    }

    // Failsafe info
    let failsafe_txt = format!(
        "(Auto-unlock dalam {}m jika tidak ada respons)",
        state.failsafe_minutes
    );
    draw_text(
        &failsafe_txt,
        card.x + 24,
        card.y + 448,
        10,
        false,
        mr,
        mg,
        mb,
    );

    // Footer
    draw_text(
        "This program was developed by Muhamad Fahmi,",
        card.x + 24,
        card.y + 470,
        10,
        false,
        mr,
        mg,
        mb,
    );
    draw_text(
        "Assistant Head of the Computer Lab.",
        card.x + 24,
        card.y + 486,
        10,
        false,
        mr,
        mg,
        mb,
    );

    EndPaint(hwnd, &ps);
}

/// Proses submit password - verifikasi dan kirim event
#[cfg(target_os = "windows")]
unsafe fn handle_submit(hwnd: HWND) {
    use crate::security::memory::zero_bytes;
    use crate::security::SecureString;
    use crate::ui::components::ctrl_id;
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetDlgItemTextW, GetWindowLongPtrW, SetDlgItemTextW, SetTimer, GWLP_USERDATA,
    };

    // Ambil pointer state dari GWLP_USERDATA
    let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
    if state_ptr == 0 {
        return;
    }
    let state = &mut *(state_ptr as *mut OverlayState);

    // Baca teks dari Edit control (UTF-16)
    let mut buf: Vec<u16> = vec![0u16; 256];
    let len = GetDlgItemTextW(hwnd, ctrl_id::ID_INPUT_PASSWORD, &mut buf) as usize;
    // Trim ke panjang aktual
    buf.truncate(len);
    // Konversi ke String (temporary)
    let s = String::from_utf16_lossy(&buf); // fallback ke empty jika konversi gagal

    // Validasi kosong lebih awal
    if s.is_empty() {
        state.status_msg = "Masukkan kata sandi terlebih dahulu.".to_string();
        state.status_is_error = true;
        let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
        // zeroize temporary buffers sebelum return
        // kosongkan buf
        buf.iter_mut().for_each(|w| *w = 0);
        // zeroize bytes from s
        let mut s_bytes = s.into_bytes();
        zero_bytes(&mut s_bytes);
        return;
    }

    // Buat SecureString dari temporary String
    // Buat SecureString dari temporary String `s`
    let secure = match SecureString::try_from_str(&s) {
        Ok(sec) => sec,
        Err(e) => {
            // Validasi gagal (mis. terlalu panjang)
            state.status_msg = e.to_string();
            state.status_is_error = true;
            let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);

            // zeroize temporaries sebelum return
            buf.iter_mut().for_each(|w| *w = 0);
            let mut s_bytes = s.into_bytes();
            zero_bytes(&mut s_bytes);
            return;
        }
    };

    // Zeroize temporary buffers asap
    buf.iter_mut().for_each(|w| *w = 0);
    let mut s_bytes = s.into_bytes();
    zero_bytes(&mut s_bytes);

    // Verifikasi menggunakan SecureString (tidak membuat salinan plaintext)
    let auth_result: AppResult<AuthStatus> = match state.auth_manager.lock() {
        Ok(mut mgr) => mgr.authenticate(&secure),
        Err(_) => {
            state.status_msg = "Error sistem autentikasi.".to_string();
            state.status_is_error = true;
            // optional: explicit zero of secure if you want to wipe now
            // let mut sec = secure; sec.explicit_zero();
            let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
            return;
        }
    };

    // Bersihkan field password di UI (kosongkan edit control)
    let empty_w = to_wide("");
    let _ = SetDlgItemTextW(hwnd, ctrl_id::ID_INPUT_PASSWORD, PCWSTR(empty_w.as_ptr()));

    // Tangani hasil autentikasi
    match auth_result {
        Ok(AuthStatus::Success) => {
            state.status_msg = "  Berhasil! Membuka akses...".to_string();
            state.status_is_error = false;
            let _ = state.event_tx.send(AppEvent::UnlockSuccess {
                trace_id: state.trace_id,
                username: state.display.username.clone(),
                unlocked_at: crate::utils::time::now_utc(),
            });
            // Tutup overlay setelah 800ms
            let _ = SetTimer(hwnd, TIMER_CLOSE_OVERLAY, 800, None);
        }
        Ok(AuthStatus::Failed) => {
            state.display.attempts += 1;
            state.status_msg = format!(
                "  Kata sandi salah. ({}/{})",
                state.display.attempts, state.display.max_attempts
            );
            state.status_is_error = true;
            let _ = state.event_tx.send(AppEvent::UnlockFailed {
                trace_id: state.trace_id,
                attempts: state.display.attempts,
                max_attempts: state.display.max_attempts,
            });
        }
        Ok(AuthStatus::LockedOut { remaining_seconds }) => {
            state.status_msg =
                format!("  Terlalu banyak percobaan. Tunggu {}s.", remaining_seconds);
            state.status_is_error = true;
        }
        Err(e) => {
            error!(error = %e, "Error verifikasi di overlay");
            state.status_msg = "Error sistem, hubungi administrator.".to_string();
            state.status_is_error = true;
        }
    }

    // Invalidate untuk redraw status
    let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);

    // SecureString akan zeroize otomatis saat drop; jika Anda ingin wipe lebih awal:
    // secure.explicit_zero();
}

/// Stub overlay untuk non-Windows
#[cfg(not(target_os = "windows"))]
fn run_overlay_stub(
    display: DisplayData,
    auth_manager: Arc<Mutex<AuthManager>>,
    event_tx: Sender<AppEvent>,
    trace_id: Uuid,
) -> AppResult<()> {
    // Kirim heartbeat untuk watchdog
    send_watchdog_heartbeat(ComponentId::UiOverlay);

    info!(%trace_id, "[SIMULASI] Overlay: proses '{}' (PID {})",
        display.process_name, display.pid);

    println!("\n╔══════════════════════════════════════════════════╗");
    println!("║         PERINGATAN KEAMANAN  [SIMULASI]          ║");
    println!("╠══════════════════════════════════════════════════╣");
    println!(
        "║ Proses  : {} (PID: {})",
        display.process_name, display.pid
    );
    println!("║ User    : {}", display.username);
    println!("║ PC      : {}", display.computer_name);
    println!("║ Waktu   : {}", display.timestamp);
    println!("╠══════════════════════════════════════════════════╣");
    println!("║ [SIMULASI] Auto-unlock dalam 3 detik...          ║");
    println!("╚══════════════════════════════════════════════════╝\n");

    std::thread::sleep(Duration::from_secs(3));

    let _ = event_tx.send(AppEvent::UnlockSuccess {
        trace_id,
        username: "SIMULATION".to_string(),
        unlocked_at: crate::utils::time::now_utc(),
    });

    Ok(())
}
