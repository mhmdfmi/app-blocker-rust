/// Penanganan input pengguna pada overlay: pembacaan password, keyboard hook.
use crate::security::memory::SecureString;
// use crate::utils::error::{AppError, AppResult};  // Uncomment jika ingin menggunakan error handling khusus

/// State input password yang aman
#[derive(Debug, Default)]
pub struct PasswordInputState {
    /// Karakter yang sudah dimasukkan (aman dari logging)
    buffer: String,
    /// Apakah field dalam keadaan dikunci (cooldown)
    locked: bool,
    /// Jumlah karakter maksimal
    max_length: usize,
}

impl PasswordInputState {
    pub fn new(max_length: usize) -> Self {
        Self {
            buffer: String::new(),
            locked: false,
            max_length,
        }
    }

    /// Tambahkan karakter ke buffer
    pub fn push_char(&mut self, c: char) -> bool {
        if self.locked || self.buffer.len() >= self.max_length {
            return false;
        }
        self.buffer.push(c);
        true
    }

    /// Hapus karakter terakhir (backspace)
    pub fn pop_char(&mut self) {
        self.buffer.pop();
    }

    /// Ambil password sebagai SecureString dan reset buffer
    pub fn take_password(&mut self) -> SecureString {
        let content = std::mem::take(&mut self.buffer);
        SecureString::try_from_str_allow_empty(&content, true).unwrap()
    }

    /// Bersihkan buffer
    pub fn clear(&mut self) {
        // Overwrite buffer secara manual sebelum drop
        for b in unsafe { self.buffer.as_bytes_mut() } {
            *b = 0;
        }
        self.buffer.clear();
    }

    /// Panjang password saat ini (untuk tampilan asterisk)
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Apakah buffer kosong
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Kunci input (selama cooldown)
    pub fn lock(&mut self) {
        self.locked = true;
    }

    /// Buka kunci input
    pub fn unlock(&mut self) {
        self.locked = false;
        self.clear();
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }
}

impl Drop for PasswordInputState {
    fn drop(&mut self) {
        self.clear();
    }
}

/// Tombol virtual key Windows yang relevan
pub mod vkeys {
    pub const VK_RETURN: u32 = 0x0D;
    pub const VK_ESCAPE: u32 = 0x1B;
    pub const VK_BACK: u32 = 0x08;
    pub const VK_F4: u32 = 0x73;
    pub const VK_DELETE: u32 = 0x2E;
}
