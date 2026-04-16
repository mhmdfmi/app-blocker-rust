#[cfg(test)]
mod state_tests {
    use crate::core::state::{AppState, State};
    use crate::core::events::AppEvent;
    #[test]
    fn test_state_transition() {
        let mut state = AppState::new();
        assert_eq!(state.current_state, State::Monitoring);
    }
}
