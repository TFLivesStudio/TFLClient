//! TFL MCP protocol contracts and authenticated private relay. No business logic.
pub mod catalog;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
pub const MAX_REQUEST: usize = 65536;
pub const MAX_RESPONSE: usize = 1024 * 1024;
#[derive(Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub port: u16,
    pub token: String,
    pub contract: u32,
}
#[derive(Serialize, Deserialize)]
pub struct Request {
    pub method: String,
    #[serde(default)]
    pub params: Value,
}
#[derive(Serialize, Deserialize)]
pub struct Response {
    pub result: Option<Value>,
    pub error: Option<String>,
}
impl Response {
    pub fn from_result(r: Result<Value, String>) -> Self {
        match r {
            Ok(v) => Self {
                result: Some(v),
                error: None,
            },
            Err(e) => Self {
                result: None,
                error: Some(e),
            },
        }
    }
}
pub fn endpoint_path() -> PathBuf {
    if let Some(root) = std::env::var_os("TFL_DATA_DIR").map(PathBuf::from) {
        if root.is_absolute() && root.is_dir() {
            return root.join(".tflclient/settings/mcp/endpoint.json");
        }
    }
    directories::UserDirs::new()
        .map(|u| u.home_dir().join(".tflclient/settings/mcp/endpoint.json"))
        .unwrap_or_else(|| PathBuf::from(".tflclient/settings/mcp/endpoint.json"))
}
pub async fn read_frame<R: tokio::io::AsyncBufRead + Unpin>(
    reader: &mut R,
    max: usize,
) -> Result<Vec<u8>, String> {
    let mut data = Vec::new();
    loop {
        let b = reader.fill_buf().await.map_err(|_| "IPC disconnected")?;
        if b.is_empty() {
            return Err("IPC disconnected; reopen TFL Client and reconnect MCP".into());
        }
        let end = b
            .iter()
            .position(|c| *c == b'\n')
            .map(|i| i + 1)
            .unwrap_or(b.len());
        if data.len() + end > max {
            return Err("IPC frame exceeds limit".into());
        }
        data.extend_from_slice(&b[..end]);
        reader.consume(end);
        if data.last() == Some(&b'\n') {
            return Ok(data);
        }
    }
}
pub async fn write_frame<W: tokio::io::AsyncWrite + Unpin, T: Serialize>(
    writer: &mut W,
    value: &T,
) -> Result<(), String> {
    let mut data = serde_json::to_vec(value).map_err(|_| "Invalid IPC data")?;
    if data.len() > MAX_RESPONSE {
        return Err("Response too large; use pagination".into());
    }
    data.push(b'\n');
    writer
        .write_all(&data)
        .await
        .map_err(|_| "IPC disconnected".into())
}
pub async fn connect(
    path: &Path,
    client: &str,
) -> Result<BufReader<tokio::net::TcpStream>, String> {
    let raw=tokio::fs::read(path).await.map_err(|_|"MCP is disabled or launcher is closed. Open TFL Client → Settings → Integrations and enable MCP.")?;
    if raw.len() > 4096 {
        return Err("Invalid MCP endpoint".into());
    }
    let endpoint: Endpoint = serde_json::from_slice(&raw).map_err(|_| "Invalid MCP endpoint")?;
    if endpoint.contract != 1 || endpoint.token.len() != 64 {
        return Err("Unsupported MCP endpoint".into());
    }
    let mut stream = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        tokio::net::TcpStream::connect((std::net::Ipv4Addr::LOCALHOST, endpoint.port)),
    )
    .await
    .map_err(|_| "Launcher connection timed out")?
    .map_err(|_| "Launcher is closed; reopen it and reconnect")?;
    write_frame(
        &mut stream,
        &serde_json::json!({"token":endpoint.token,"client":client}),
    )
    .await?;
    Ok(BufReader::new(stream))
}
/// Protect transport secrets on disk. Never pass tokens in arguments or URLs.
pub fn protect(path: &Path, directory: bool) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            path,
            std::fs::Permissions::from_mode(if directory { 0o700 } else { 0o600 }),
        )?;
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        #[link(name = "advapi32")]
        unsafe extern "system" {
            fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
                s: *const u16,
                r: u32,
                p: *mut *mut core::ffi::c_void,
                size: *mut u32,
            ) -> i32;
            fn SetFileSecurityW(p: *const u16, info: u32, s: *mut core::ffi::c_void) -> i32;
        }
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn LocalFree(p: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
        }
        let s: Vec<u16> = "D:P(A;;FA;;;OW)".encode_utf16().chain(Some(0)).collect();
        let p: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut sd = std::ptr::null_mut();
        // OWNER RIGHTS SID: only this object's owner. Protected DACL excludes inherited grants.
        unsafe {
            if ConvertStringSecurityDescriptorToSecurityDescriptorW(
                s.as_ptr(),
                1,
                &mut sd,
                std::ptr::null_mut(),
            ) == 0
            {
                return Err(std::io::Error::last_os_error());
            }
            let ok = SetFileSecurityW(p.as_ptr(), 0x80000004, sd);
            LocalFree(sd);
            if ok == 0 {
                return Err(std::io::Error::last_os_error());
            }
        }
        let _ = directory;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use catalog::*;
    use serde_json::json;
    #[test]
    fn catalog_is_unique_and_schemas_are_strict() {
        let defs = definitions();
        let mut names = std::collections::HashSet::new();
        assert!(defs.len() >= 28);
        for d in defs {
            assert!(names.insert(d.name));
            assert_eq!(d.schema["additionalProperties"], false);
            assert!(validate(&d.schema, &json!({"run_shell":"oops"})).is_err());
        }
    }
    #[test]
    fn schema_rejects_extremes_and_unknown_actions() {
        let defs = definitions();
        let d = defs
            .iter()
            .find(|d| d.name == "tfl_set_instance_memory")
            .unwrap();
        assert!(
            validate(
                &d.schema,
                &json!({"instance_id":"x","min_mb":0,"max_mb":99999999})
            )
            .is_err()
        );
        let p = defs.iter().find(|d| d.name == "tfl_plan_action").unwrap();
        assert!(validate(&p.schema, &json!({"instance_id":"x","action":"run_shell"})).is_err());
    }
    #[tokio::test]
    async fn frame_is_bounded_and_disconnect_is_error() {
        let mut r = std::io::Cursor::new(vec![b'x'; 100]);
        assert!(read_frame(&mut r, 50).await.is_err());
        let mut r = std::io::Cursor::new(b"{}\n");
        assert_eq!(read_frame(&mut r, 50).await.unwrap(), b"{}\n");
        assert!(read_frame(&mut r, 50).await.is_err());
    }
}
