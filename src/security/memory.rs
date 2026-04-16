//! Memory Security Module
use std::ptr::write_volatile;

pub struct SecureBuffer {
    data: Vec<u8>,
}

impl SecureBuffer {
    pub fn new(size: usize) -> Self {
        Self { data: vec![0u8; size] }
    }
    
    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data }
    }
    
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }
    
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    fn zero(&mut self) {
        for byte in &mut self.data {
            unsafe { write_volatile(byte, 0); }
        }
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        self.zero();
    }
}

pub fn secure_zero_memory(slice: &mut [u8]) {
    let len = slice.len();
    if len == 0 { return; }
    
    for i in 0..len {
        unsafe { write_volatile(&mut slice[i], 0); }
    }
    
    for byte in slice.iter() {
        if *byte != 0 {
            for i in 0..len {
                unsafe { write_volatile(&mut slice[i], 0); }
            }
            break;
        }
    }
}

pub struct SecureString(String);

impl SecureString {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    
    pub fn len(&self) -> usize {
        self.0.len()
    }
    
    pub fn masked(&self) -> String {
        if self.0.len() <= 4 { "****".to_string() }
        else { format!("{}****", &self.0[..2]) }
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        let mut data = self.0.as_bytes().to_vec();
        secure_zero_memory(&mut data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_secure_buffer() {
        let mut buf = SecureBuffer::new(10);
        buf.as_mut_slice()[0] = 0xFF;
        assert_eq!(buf.len(), 10);
    }
}
