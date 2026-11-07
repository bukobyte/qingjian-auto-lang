//! 私有 Unix socket 状态发布器；连接只返回快照，不能执行命令。

use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, FileTypeExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use qingjian_auto_language::RuntimeSnapshot;

use super::{IO_TIMEOUT, MAX_STATUS_BYTES};

pub struct StatusServer {
    snapshot: Arc<Mutex<RuntimeSnapshot>>,

    stop: Arc<AtomicBool>,

    worker: Option<JoinHandle<()>>,

    path: PathBuf,
}

impl StatusServer {
    pub fn start(path: &Path) -> io::Result<Self> {
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "socket needs parent directory")
        })?;
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(parent)?;
        let metadata = fs::symlink_metadata(parent)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "status directory must not be a symlink",
            ));
        }
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
        if let Ok(metadata) = fs::symlink_metadata(path) {
            if !metadata.file_type().is_socket() {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "status endpoint is not a socket",
                ));
            }
            match UnixStream::connect(path) {
                Ok(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::AddrInUse,
                        "automatic language status already running",
                    ));
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::ConnectionRefused | io::ErrorKind::NotFound
                    ) =>
                {
                    fs::remove_file(path)?
                }
                Err(error) => return Err(error),
            }
        }
        let listener = UnixListener::bind(path)?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        listener.set_nonblocking(true)?;
        let snapshot = Arc::new(Mutex::new(RuntimeSnapshot::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let worker_snapshot = Arc::clone(&snapshot);
        let worker_stop = Arc::clone(&stop);
        let worker = thread::Builder::new()
            .name("auto-language-status".into())
            .spawn(move || {
                while !worker_stop.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((mut stream, _)) => {
                            let _ = stream.set_write_timeout(Some(IO_TIMEOUT));
                            let bytes = worker_snapshot
                                .lock()
                                .ok()
                                .and_then(|state| serde_json::to_vec(&*state).ok());
                            if let Some(bytes) =
                                bytes.filter(|bytes| bytes.len() as u64 <= MAX_STATUS_BYTES)
                            {
                                let _ = stream.write_all(&bytes);
                            }
                        }
                        Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(20))
                        }
                        Err(_) => break,
                    }
                }
            })?;
        Ok(Self {
            snapshot,
            stop,
            worker: Some(worker),
            path: path.to_owned(),
        })
    }

    pub fn publish(&self, snapshot: RuntimeSnapshot) {
        if let Ok(mut current) = self.snapshot.lock() {
            *current = snapshot;
        }
    }
}

impl Drop for StatusServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        let _ = fs::remove_file(&self.path);
    }
}
