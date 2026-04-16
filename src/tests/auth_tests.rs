#[cfg(test)]
mod auth_tests {
    use super::*;
    use crate::security::auth::Authenticator;
    #[test]
    fn test_auth_success() {
        let auth = Authenticator::new(3, 60);
        let hash = Authenticator::hash_password("test123").unwrap();
        assert!(auth.verify("test123", &hash).unwrap());
    }
    #[test]
    fn test_auth_failure() {
        let auth = Authenticator::new(3, 60);
        let hash = Authenticator::hash_password("test123").unwrap();
        assert!(!auth.verify("wrong", &hash).unwrap());
    }
}
