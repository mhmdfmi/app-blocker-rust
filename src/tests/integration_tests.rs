/// Integration tests - skenario kompleks: channel failure, mutex contention, kill loop.
#[cfg(test)]
mod tests {
    use app_blocker_lib::{
        core::{
            events::AppEvent,
            state::{AppState, StateManager},
        },
        security::auth::{Argon2AuthService, AuthManager, AuthStatus, DEFAULT_PASSWORD},
        system::process::{ProcessInfo, ProcessService, WindowsProcessService},
        utils::error::AppError,
    };
    use std::sync::{mpsc, Arc, Mutex};
    use std::time::Duration;

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn make_auth() -> AuthManager {
        let (svc, _) = Argon2AuthService::with_default_password().unwrap();
        AuthManager::new(Box::new(svc), 5, 60)
    }

    fn make_proc(name: &str) -> ProcessInfo {
        ProcessInfo {
            pid: 1234,
            name: name.to_string(),
            exe_path: None,
            username: Some("test".to_string()),
            cpu_usage: 10.0,
            status: "Run".to_string(),
        }
    }

    // ── Test: Channel disconnect ──────────────────────────────────────────────

    /// Test: kirim ke channel yang sudah di-drop menghasilkan error
    #[test]
    fn test_channel_disconnect_detected() {
        let (tx, rx) = mpsc::channel::<AppEvent>();
        drop(rx); // Putuskan receiver

        let result = tx.send(AppEvent::ConfigReloaded);
        assert!(result.is_err(), "Kirim ke channel mati harus error");
    }

    /// Test: receiver yang terputus mengembalikan error saat recv
    #[test]
    fn test_channel_recv_after_sender_dropped() {
        let (tx, rx) = mpsc::channel::<AppEvent>();
        drop(tx);

        let result = rx.recv_timeout(Duration::from_millis(100));
        assert!(result.is_err(), "Receive dari channel mati harus error");
    }

    // ── Test: State machine concurrent access ─────────────────────────────────

    /// Test: state manager aman diakses dari banyak thread (thread safety)
    #[test]
    fn test_state_manager_thread_safe() {
        let mgr = Arc::new(StateManager::new());
        let mut handles = vec![];

        // 10 thread baca state secara concurrent
        for _ in 0..10 {
            let m = Arc::clone(&mgr);
            handles.push(std::thread::spawn(move || {
                for _ in 0..100 {
                    let _ = m.current_state();
                    let _ = m.read_data(|d| d.consecutive_errors);
                }
            }));
        }

        for h in handles {
            h.join().expect("Thread tidak boleh panic");
        }

        // State harus masih valid
        assert_eq!(mgr.current_state().unwrap(), AppState::Monitoring);
    }

    /// Test: transisi concurrent tidak menyebabkan state corruption
    #[test]
    fn test_sequential_state_transitions_correct() {
        let mgr = StateManager::new();

        // Simulasikan siklus penuh
        for _ in 0..5 {
            mgr.transition_to(AppState::Blocking,   "cycle").unwrap();
            mgr.transition_to(AppState::Locked,     "cycle").unwrap();
            mgr.transition_to(AppState::Recovering, "cycle").unwrap();
            mgr.transition_to(AppState::Monitoring, "cycle").unwrap();
        }

        assert_eq!(mgr.current_state().unwrap(), AppState::Monitoring);
    }

    // ── Test: Mutex contention ────────────────────────────────────────────────

    /// Test: AuthManager aman diakses concurrent via Arc<Mutex>
    #[test]
    fn test_auth_manager_concurrent_access() {
        let mgr = Arc::new(Mutex::new(make_auth()));
        let mut handles = vec![];

        for _ in 0..5 {
            let m = Arc::clone(&mgr);
            handles.push(std::thread::spawn(move || {
                if let Ok(mut auth) = m.lock() {
                    // Authenticate dengan password salah tidak panic
                    let _ = auth.authenticate("wrong_password");
                }
            }));
        }

        for h in handles {
            h.join().expect("Thread auth tidak boleh panic");
        }
    }

    // ── Test: Kill loop safety ────────────────────────────────────────────────

    /// Test: protected process tidak bisa di-kill (mode simulasi)
    #[test]
    fn test_kill_protected_process_rejected() {
        let svc = WindowsProcessService::new(true); // simulasi
        let result = svc.kill_process(4, "System");
        assert!(result.is_err(), "Kill process System (PID 4) harus ditolak");
    }

    /// Test: kill proses dengan PID 0 ditolak
    #[test]
    fn test_kill_pid_zero_rejected() {
        let svc = WindowsProcessService::new(true);
        let result = svc.kill_process(0, "idle");
        assert!(result.is_err(), "Kill PID 0 harus ditolak");
    }

    /// Test: is_protected case-insensitive
    #[test]
    fn test_protected_check_case_insensitive() {
        let svc = WindowsProcessService::new(true);
        assert!(svc.is_protected("WINLOGON.EXE"), "Case insensitive harus bekerja");
        assert!(svc.is_protected("Explorer.exe"), "Case insensitive harus bekerja");
    }

    // ── Test: Auth rate limiting integration ─────────────────────────────────

    /// Test: exponential backoff lockout bekerja setelah max_attempts
    #[test]
    fn test_auth_lockout_integration() {
        let (svc, _) = Argon2AuthService::with_default_password().unwrap();
        let mut mgr = AuthManager::new(Box::new(svc), 3, 30);

        // 3 gagal → lockout
        for i in 0..3 {
            let res = mgr.authenticate("salah_semua").unwrap();
            if i < 2 {
                assert_eq!(res, AuthStatus::Failed);
            }
        }

        // Percobaan berikutnya → LockedOut
        let res = mgr.authenticate("apapun").unwrap();
        assert!(
            matches!(res, AuthStatus::LockedOut { .. }),
            "Harus LockedOut setelah 3 percobaan gagal"
        );
    }

    /// Test: Auth berhasil dengan password benar setelah 1 gagal
    #[test]
    fn test_auth_success_after_failure() {
        let (svc, _) = Argon2AuthService::with_default_password().unwrap();
        let mut mgr = AuthManager::new(Box::new(svc), 5, 30);

        assert_eq!(mgr.authenticate("salah").unwrap(), AuthStatus::Failed);
        assert_eq!(mgr.failed_attempts(), 1);

        assert_eq!(
            mgr.authenticate(DEFAULT_PASSWORD).unwrap(),
            AuthStatus::Success
        );
        assert_eq!(mgr.failed_attempts(), 0, "Harus reset setelah sukses");
    }

    // ── Test: Event system ────────────────────────────────────────────────────

    /// Test: semua variant AppEvent bisa dibuat dan dikirim via channel
    #[test]
    fn test_all_events_sendable() {
        let (tx, rx) = mpsc::channel::<AppEvent>();

        let events = vec![
            AppEvent::ConfigReloaded,
            AppEvent::DisableFlagDetected,
            AppEvent::EnterSafeMode { reason: "test".to_string() },
            AppEvent::ShutdownRequested { reason: "test".to_string() },
        ];

        for event in events {
            tx.send(event).expect("Event harus bisa dikirim");
        }

        let mut received = 0;
        while let Ok(_) = rx.recv_timeout(Duration::from_millis(10)) {
            received += 1;
        }
        assert_eq!(received, 4, "Semua event harus diterima");
    }

    /// Test: event name() tidak panic untuk semua variant
    #[test]
    fn test_event_name_all_variants() {
        let trace = uuid::Uuid::new_v4();
        let proc = make_proc("test.exe");

        let events: Vec<AppEvent> = vec![
            AppEvent::ProcessDetected {
                trace_id: trace, info: proc.clone(),
                score: 90, detected_at: app_blocker_lib::utils::time::now_utc(),
            },
            AppEvent::ProcessBlocked {
                trace_id: trace, pid: 1234, name: "test.exe".into(),
                killed_at: app_blocker_lib::utils::time::now_utc(),
            },
            AppEvent::ProcessBlockFailed {
                trace_id: trace, pid: 1234,
                name: "test.exe".into(), reason: "err".into(),
            },
            AppEvent::OverlayRequested {
                trace_id: trace, info: proc,
                triggered_at: app_blocker_lib::utils::time::now_utc(),
            },
            AppEvent::UnlockSuccess {
                trace_id: trace, username: "user".into(),
                unlocked_at: app_blocker_lib::utils::time::now_utc(),
            },
            AppEvent::UnlockFailed { trace_id: trace, attempts: 1, max_attempts: 5 },
            AppEvent::ShutdownRequested { reason: "test".into() },
            AppEvent::EmergencyUnlock { trace_id: trace },
            AppEvent::EnterSafeMode { reason: "test".into() },
            AppEvent::ConfigReloaded,
            AppEvent::DisableFlagDetected,
        ];

        for e in &events {
            // name() tidak boleh panic
            let _ = e.name();
            let _ = e.is_critical();
        }
    }
}
