/// Pengelolaan memori aman - menghapus data sensitif saat tidak digunakan.
/// Menggunakan zeroize untuk mencegah data sensitif tertinggal di memori.
/// Buffer sensitif yang otomatis di-zero saat drop
use crate::utils::error::{AppError, AppResult};
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// SecureBuffer: raw bytes, zeroized on drop
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecureBuffer {
    inner: Vec<u8>,
}

impl SecureBuffer {
    pub fn try_new(data: Vec<u8>) -> AppResult<Self> {
        if data.is_empty() {
            return Err(AppError::Validation("Buffer kosong".into()));
        }
        if data.len() > 16 * 1024 {
            return Err(AppError::Validation("Buffer terlalu besar".into()));
        }
        Ok(Self { inner: data })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn explicit_zero(&mut self) {
        self.inner.zeroize();
    }

    /// Consume self and return owned bytes safely.
    /// Implementation uses `mut self` + `mem::take` to avoid moving a field out of a Drop type.
    pub fn into_bytes(mut self) -> Vec<u8> {
        // swap inner with empty Vec and return the original bytes
        std::mem::take(&mut self.inner)
        // `self` is dropped here; Drop will zeroize the now-empty Vec (no secret left)
    }
}

impl fmt::Debug for SecureBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecureBuffer([{} bytes, REDACTED])", self.inner.len())
    }
}

/// SecureString: store UTF-8 as bytes, zeroized on drop
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecureString {
    inner: Vec<u8>,
}

impl SecureString {
    pub fn try_from_str(s: &str) -> AppResult<Self> {
        Self::try_from_str_allow_empty(s, true)
    }

    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Buat SecureString, optionally allow empty
    pub fn try_from_str_allow_empty(s: &str, allow_empty: bool) -> AppResult<Self> {
        let bytes = s.as_bytes();
        if bytes.is_empty() && !allow_empty {
            return Err(AppError::Validation("Password kosong".into()));
        }
        if bytes.len() > 4096 {
            return Err(AppError::Validation("Password terlalu panjang".into()));
        }
        Ok(Self {
            inner: bytes.to_vec(),
        })
    }

    /// Run closure with temporary cleartext reference
    pub fn with_cleartext<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&str) -> R,
    {
        // assume valid UTF-8; upstream should validate if necessary
        let s = std::str::from_utf8(&self.inner).unwrap_or_default();
        f(s)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn explicit_zero(&mut self) {
        self.inner.zeroize();
    }

    /// Consume self and return owned bytes safely.
    pub fn into_bytes(mut self) -> Vec<u8> {
        std::mem::take(&mut self.inner)
    }

    /// Consume self and attempt to return owned String.
    /// Returns None if bytes are not valid UTF-8.
    pub fn into_string(mut self) -> Option<String> {
        // take bytes out safely
        let bytes = std::mem::take(&mut self.inner);
        String::from_utf8(bytes).ok()
        // `self` dropped here; inner is empty so Drop zeroize is harmless
    }
}

impl fmt::Debug for SecureString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecureString([REDACTED])")
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
