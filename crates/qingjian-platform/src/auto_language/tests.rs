//! 本地状态通道的快照、生命周期与文件权限。

use std::os::unix::fs::PermissionsExt;

use crate::RuntimeSnapshot;

use super::{StatusServer, query_status};

#[test]
fn auto_language_status_round_trip_is_private_and_cleans_up() {
    let dir = std::env::temp_dir().join(format!("qj-status-{}", std::process::id()));
    let path = dir.join("status.sock");
    let server = StatusServer::start(&path).unwrap();
    let expected = RuntimeSnapshot {
        session_id: "test".into(),
        enabled: true,
        ..RuntimeSnapshot::default()
    };
    server.publish(expected.clone());
    assert_eq!(query_status(&path).unwrap(), expected);
    assert_eq!(
        std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert!(StatusServer::start(&path).is_err());
    drop(server);
    assert!(!path.exists());
    assert!(query_status(&path).is_err());
    std::fs::remove_dir(dir).unwrap();
}

#[test]
fn auto_language_status_rejects_non_socket_and_incompatible_protocol() {
    let dir = std::env::temp_dir().join(format!("qj-status-version-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("status.sock");
    std::fs::write(&path, "keep").unwrap();
    assert!(StatusServer::start(&path).is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "keep");
    std::fs::remove_file(&path).unwrap();
    let server = StatusServer::start(&path).unwrap();
    server.publish(RuntimeSnapshot {
        version: 999,
        ..RuntimeSnapshot::default()
    });
    assert!(query_status(&path).is_err());
    drop(server);
    std::fs::remove_dir(dir).unwrap();
}
