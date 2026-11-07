//! 有界查询输入法状态，不在输入按键路径调用。

use std::io::{self, Read};
use std::os::unix::net::UnixStream;
use std::path::Path;

use qingjian_auto_language::{RuntimeSnapshot, SNAPSHOT_VERSION};

use super::{IO_TIMEOUT, MAX_STATUS_BYTES};

pub fn query_status(path: &Path) -> io::Result<RuntimeSnapshot> {
    let stream = UnixStream::connect(path)?;
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    let mut bytes = Vec::new();
    stream.take(MAX_STATUS_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_STATUS_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "automatic language status exceeds size limit",
        ));
    }
    let snapshot: RuntimeSnapshot = serde_json::from_slice(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if snapshot.version != SNAPSHOT_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported automatic language status version",
        ));
    }
    Ok(snapshot)
}
