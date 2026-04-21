<<<<<<< HEAD
/// Unit tests state machine.
#[cfg(test)]
mod tests {
    use app_blocker_lib::core::state::{AppState, StateManager};

    #[test]
    fn test_valid_transitions() {
        let cases = vec![
            (AppState::Monitoring, AppState::Blocking),
            (AppState::Blocking,   AppState::Locked),
            (AppState::Locked,     AppState::Recovering),
            (AppState::Recovering, AppState::Monitoring),
            (AppState::Monitoring, AppState::SafeMode),
            (AppState::Locked,     AppState::SafeMode),
            (AppState::SafeMode,   AppState::Monitoring),
        ];
        for (from, to) in cases {
            assert!(from.can_transition_to(&to),
                "Transisi {from} -> {to} harus valid");
        }
    }

    #[test]
    fn test_forbidden_transitions() {
        let cases = vec![
            (AppState::Monitoring, AppState::Locked),
            (AppState::Locked,     AppState::Monitoring),
            (AppState::Recovering, AppState::Locked),
            (AppState::Blocking,   AppState::Recovering),
        ];
        for (from, to) in &cases {
            assert!(!from.can_transition_to(to),
                "Transisi {from} -> {to} harus dilarang");
        }
    }

    #[test]
    fn test_state_manager_happy_path() {
        let mgr = StateManager::new();
        assert_eq!(mgr.current_state().unwrap(), AppState::Monitoring);
        mgr.transition_to(AppState::Blocking,   "test").unwrap();
        mgr.transition_to(AppState::Locked,     "test").unwrap();
        mgr.transition_to(AppState::Recovering, "test").unwrap();
        mgr.transition_to(AppState::Monitoring, "test").unwrap();
        assert_eq!(mgr.current_state().unwrap(), AppState::Monitoring);
    }

    #[test]
    fn test_invalid_transition_returns_error() {
        let mgr = StateManager::new();
        assert!(mgr.transition_to(AppState::Locked, "bad").is_err(),
            "Monitoring -> Locked harus error");
    }

    #[test]
    fn test_force_safe_mode() {
        let mgr = StateManager::new();
        mgr.transition_to(AppState::Blocking, "test").unwrap();
        mgr.force_safe_mode("emergency").unwrap();
        assert_eq!(mgr.current_state().unwrap(), AppState::SafeMode);
        let disabled = mgr.read_data(|d| d.blocking_disabled).unwrap();
        assert!(disabled);
    }

    #[test]
    fn test_reset_data() {
        let mgr = StateManager::new();
        mgr.update_data(|d| {
            d.overlay_active     = true;
            d.blocked_pid        = Some(1234);
            d.consecutive_errors = 3;
        }).unwrap();
        mgr.reset_data().unwrap();
        let (ov, pid, errs) = mgr.read_data(|d|
            (d.overlay_active, d.blocked_pid, d.consecutive_errors)).unwrap();
        assert!(!ov);
        assert!(pid.is_none());
        assert_eq!(errs, 0);
    }

    #[test]
    fn test_state_properties() {
        assert!(AppState::Monitoring.allows_blocking());
        assert!(!AppState::Locked.allows_blocking());
        assert!(!AppState::SafeMode.allows_blocking());
        assert!(AppState::Locked.requires_overlay());
        assert!(!AppState::Monitoring.requires_overlay());
        assert!(AppState::SafeMode.is_safe_mode());
        assert!(!AppState::Monitoring.is_safe_mode());
=======
#[cfg(test)]
mod state_tests {
    use crate::core::state::{AppState, State};
    use crate::core::events::AppEvent;
    #[test]
    fn test_state_transition() {
        let mut state = AppState::new();
        assert_eq!(state.current_state, State::Monitoring);
>>>>>>> bce0345919f371d153ccb843f2ddbfb5e8695c5f
    }
}
