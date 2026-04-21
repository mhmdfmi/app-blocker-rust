<<<<<<< HEAD
/// Unit tests sistem autentikasi Argon2.
#[cfg(test)]
mod tests {
    use app_blocker_lib::security::auth::{
        Argon2AuthService, AuthManager, AuthService, AuthStatus, DEFAULT_PASSWORD,
    };

    #[test]
    fn test_hash_not_plaintext() {
        let svc = Argon2AuthService::new(String::new()).unwrap();
        let hash = svc.hash_password("rahasia123").unwrap();
        assert!(!hash.contains("rahasia123"));
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_verify_correct_password() {
        let svc = Argon2AuthService::new(String::new()).unwrap();
        let hash = svc.hash_password("Admin12345!").unwrap();
        let svc2 = Argon2AuthService::new(hash).unwrap();
        assert!(svc2.verify_password("Admin12345!").unwrap());
    }

    #[test]
    fn test_verify_wrong_password() {
        let svc = Argon2AuthService::new(String::new()).unwrap();
        let hash = svc.hash_password("Admin12345!").unwrap();
        let svc2 = Argon2AuthService::new(hash).unwrap();
        assert!(!svc2.verify_password("salah").unwrap());
    }

    #[test]
    fn test_lockout_after_max_attempts() {
        let svc = Argon2AuthService::new(String::new()).unwrap();
        let hash = svc.hash_password("benar123!").unwrap();
        let svc2 = Argon2AuthService::new(hash).unwrap();
        let mut mgr = AuthManager::new(Box::new(svc2), 3, 60);
        for _ in 0..3 { let _ = mgr.authenticate("salah"); }
        assert!(matches!(mgr.authenticate("salah").unwrap(),
            AuthStatus::LockedOut { .. }));
    }

    #[test]
    fn test_reset_after_success() {
        let svc = Argon2AuthService::new(String::new()).unwrap();
        let hash = svc.hash_password("benar123!").unwrap();
        let svc2 = Argon2AuthService::new(hash).unwrap();
        let mut mgr = AuthManager::new(Box::new(svc2), 5, 60);
        let _ = mgr.authenticate("salah");
        let _ = mgr.authenticate("salah");
        assert_eq!(mgr.failed_attempts(), 2);
        assert_eq!(mgr.authenticate("benar123!").unwrap(), AuthStatus::Success);
        assert_eq!(mgr.failed_attempts(), 0);
    }

    #[test]
    fn test_hash_uniqueness() {
        let svc = Argon2AuthService::new(String::new()).unwrap();
        let h1 = svc.hash_password("sama123").unwrap();
        let h2 = svc.hash_password("sama123").unwrap();
        assert_ne!(h1, h2, "Salt random harus hasilkan hash berbeda");
    }

    #[test]
    fn test_default_password_flow() {
        let (svc, hash) = Argon2AuthService::with_default_password().unwrap();
        assert!(!hash.is_empty());
        assert!(svc.verify_password(DEFAULT_PASSWORD).unwrap());
=======
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
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
    }
}
