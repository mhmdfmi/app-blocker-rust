/// Unit tests untuk engine deteksi dan integrasi.
#[cfg(test)]
mod tests {
    use app_blocker_lib::{
        config::settings::{AppConfig, BlockedApp},
        detection::DetectionEngine,
        system::process::{ProcessInfo, ProcessService, WindowsProcessService},
    };

    fn make_proc(name: &str, cpu: f32, exe: Option<&str>) -> ProcessInfo {
        ProcessInfo {
            pid:      9999,
            name:     name.to_string(),
            exe_path: exe.map(String::from),
            username: Some("TestUser".to_string()),
            cpu_usage: cpu,
            status:   "Run".to_string(),
        }
    }

    fn cfg_with_blacklist(names: &[&str]) -> AppConfig {
        let mut cfg = AppConfig::default();
        cfg.blocking.blacklist = names.iter().map(|n| BlockedApp {
            name:          n.to_string(),
            process_names: vec![n.to_string()],
            paths:         vec![],
            description:   String::new(),
        }).collect();
        cfg.schedule.enabled = false; // selalu aktif saat test
        cfg
    }

    #[test]
    fn test_blacklisted_process_detected() {
        let cfg = cfg_with_blacklist(&["badgame.exe"]);
        let mut engine = DetectionEngine::new(&cfg).unwrap();
        let proc = make_proc("badgame.exe", 50.0, Some(r"C:\Games\badgame.exe"));
        let res  = engine.detect(&proc).unwrap();
        assert!(res.is_some(), "Proses blacklist harus terdeteksi");
        assert!(res.unwrap().score >= cfg.blocking.score_threshold);
    }

    #[test]
    fn test_safe_process_not_detected() {
        let cfg  = cfg_with_blacklist(&["badgame.exe"]);
        let mut engine = DetectionEngine::new(&cfg).unwrap();
        let proc = make_proc("notepad.exe", 1.0, Some(r"C:\Windows\notepad.exe"));
        let res  = engine.detect(&proc).unwrap();
        assert!(res.is_none(), "Proses aman tidak boleh terdeteksi");
    }

    #[test]
    fn test_whitelist_overrides_blacklist() {
        let mut cfg = cfg_with_blacklist(&["allowed.exe"]);
        cfg.blocking.whitelist = vec!["allowed.exe".to_string()];
        let mut engine = DetectionEngine::new(&cfg).unwrap();
        let proc = make_proc("allowed.exe", 80.0, Some(r"C:\Games\allowed.exe"));
        assert!(engine.detect(&proc).unwrap().is_none(), "Whitelist harus override blacklist");
    }

    #[test]
    fn test_case_insensitive_detection() {
        let cfg  = cfg_with_blacklist(&["RobloxPlayerBeta.exe"]);
        let mut engine = DetectionEngine::new(&cfg).unwrap();
        let proc = make_proc("robloxplayerbeta.exe", 50.0, None);
        assert!(engine.detect(&proc).unwrap().is_some(), "Deteksi harus case-insensitive");
    }

    #[test]
    fn test_usb_path_game_detected() {
        let mut cfg = cfg_with_blacklist(&["steam.exe"]);
        cfg.blocking.blacklist[0].process_names = vec!["steam.exe".to_string()];
        let mut engine = DetectionEngine::new(&cfg).unwrap();
        let proc = make_proc("steam.exe", 40.0, Some(r"E:\Steam\steam.exe"));
        assert!(engine.detect(&proc).unwrap().is_some(), "Game dari USB harus terdeteksi");
    }

    #[test]
    fn test_protected_processes_recognized() {
        let svc = WindowsProcessService::new(true);
        for name in &["winlogon.exe","csrss.exe","lsass.exe","explorer.exe","smss.exe"] {
            assert!(svc.is_protected(name), "'{name}' harus protected");
        }
    }

    #[test]
    fn test_non_protected_game_not_protected() {
        let svc = WindowsProcessService::new(true);
        assert!(!svc.is_protected("steam.exe"),  "steam.exe bukan protected");
        assert!(!svc.is_protected("badgame.exe"),"badgame.exe bukan protected");
    }
}
