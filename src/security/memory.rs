/// Pengelolaan memori aman - menghapus data sensitif saat tidak digunakan.
/// Menggunakan zeroize untuk mencegah data sensitif tertinggal di memori.
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Buffer sensitif yang otomatis di-zero saat drop
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecureBuffer(Vec<u8>);

impl SecureBuffer {
    /// Buat buffer aman dari bytes
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    /// Buat buffer aman dari string
    pub fn from_str(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }

    /// Akses data sebagai slice (read-only)
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Dapatkan panjang buffer
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Periksa apakah buffer kosong
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// String sensitif yang otomatis di-zero saat drop
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecureString(String);

impl SecureString {
    /// Buat SecureString dari String (mengambil ownership)
    pub fn new(s: String) -> Self {
        Self(s)
    }

    /// Akses string sebagai &str
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Periksa apakah string kosong
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Debug for SecureString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecureString([DISEMBUNYIKAN])")
    }
}

impl std::fmt::Debug for SecureBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecureBuffer([{} bytes, DISEMBUNYIKAN])", self.0.len())
    }
}

/// Zero buffer secara eksplisit tanpa wrapper
pub fn zero_bytes(buf: &mut [u8]) {
    buf.zeroize();
}

/// Zero string secara eksplisit
pub fn zero_string(s: &mut String) {
    s.zeroize();
}
