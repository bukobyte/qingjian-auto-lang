//! MRU 的数量、去重与元信息刷新。

use crate::{AppInfo, RecentApps};

fn app(id: &str) -> AppInfo {
    AppInfo {
        app_id: id.into(),
        display_name: id.into(),
        bundle_path: None,
    }
}

#[test]
fn recency_moves_existing_app_and_evicts_oldest() {
    let mut recent = RecentApps::default();
    for i in 0..11 {
        recent.record(app(&format!("app{i}")));
    }
    assert_eq!(recent.as_slice().len(), 10);
    assert_eq!(recent.as_slice()[0].app_id, "app10");
    assert!(!recent.as_slice().iter().any(|app| app.app_id == "app0"));
    let mut changed = app("APP5");
    changed.display_name = "New name".into();
    recent.record(changed);
    assert_eq!(recent.as_slice().len(), 10);
    assert_eq!(recent.as_slice()[0].display_name, "New name");
    assert_eq!(
        recent
            .as_slice()
            .iter()
            .filter(|app| app.app_id.eq_ignore_ascii_case("app5"))
            .count(),
        1
    );
    recent.record(app("  "));
    assert_eq!(recent.as_slice().len(), 10);
}
