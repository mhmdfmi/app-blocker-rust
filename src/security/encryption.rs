/// Modul enkripsi dan hash untuk verifikasi integritas.
/// Menggunakan SHA-256 untuk hash file dan data.
use crate::utils::error::{AppError, AppResult};
use sha2::{Digest, Sha256};
use std::path::Path;
use tracing::debug;

/// Hitung hash SHA-256 dari konten string
pub fn hash_string(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}

/// Hitung hash SHA-256 dari bytes
pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Hitung hash SHA-256 dari file
pub fn hash_file(path: &Path) -> AppResult<String> {
    let content = std::fs::read(path)
        .map_err(|e| AppError::io(format!("Baca file untuk hash: {}", path.display()), e))?;
    let hash = hash_bytes(&content);
    debug!(
        path = %path.display(),
        hash = %&hash[..16],
        "Hash file dihitung"
    );
    Ok(hash)
}

/// Verifikasi hash file terhadap hash yang tersimpan
pub fn verify_file_hash(path: &Path, expected_hash: &str) -> AppResult<bool> {
    let actual = hash_file(path)?;
    Ok(actual == expected_hash)
}

/// Hasilkan hash biner dari executable yang sedang berjalan (self-check)
pub fn hash_self() -> AppResult<String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| AppError::System(format!("Gagal mendapatkan path executable: {e}")))?;
    hash_file(&exe_path)
}

/// Bandingkan dua hash secara constant-time untuk mencegah timing attack
pub fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    // XOR setiap byte - tidak short-circuit
    a.bytes()
        .zip(b.bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}
