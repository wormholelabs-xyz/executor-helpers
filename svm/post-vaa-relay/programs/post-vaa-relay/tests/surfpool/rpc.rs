//! Hand-rolled JSON-RPC over HTTP/1.1 to a localhost surfnet. `solana-client`
//! does not expose the `surfnet_*` cheatcodes.

use std::{
    io::{Read, Write},
    net::TcpStream,
    thread,
    time::Duration,
};

use serde_json::{json, Value};

const POLL_INTERVAL: Duration = Duration::from_millis(200);
const RPC_RETRIES: usize = 3;

pub fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

pub fn rpc_call(url: &str, method: &str, params: Value) -> Value {
    let mut last = String::new();
    for _ in 0..RPC_RETRIES {
        match try_rpc(url, method, params.clone()) {
            Ok(v) => return v,
            Err(e) => last = e,
        }
        thread::sleep(POLL_INTERVAL);
    }
    panic!("RPC {method} failed after {RPC_RETRIES} attempts: {last}");
}

/// One JSON-RPC POST over plain HTTP/1.1 to localhost. Handles `Content-Length`
/// and chunked bodies.
pub fn try_rpc(url: &str, method: &str, params: Value) -> Result<Value, String> {
    let host_port = url
        .strip_prefix("http://")
        .ok_or("only http:// URLs")?
        .trim_end_matches('/');
    let body = json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).to_string();
    let mut stream =
        TcpStream::connect(host_port).map_err(|e| format!("connect {host_port}: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(60)))
        .map_err(|e| e.to_string())?;
    let request = format!(
        "POST / HTTP/1.1\r\nHost: {host_port}\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("write: {e}"))?;
    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .map_err(|e| format!("read: {e}"))?;
    let split = find_subslice(&raw, b"\r\n\r\n").ok_or("no HTTP header terminator")?;
    let header = String::from_utf8_lossy(&raw[..split]).to_ascii_lowercase();
    let body = &raw[split + 4..];
    let body = if header.contains("transfer-encoding: chunked") {
        decode_chunked(body)?
    } else {
        body.to_vec()
    };
    serde_json::from_slice(&body).map_err(|e| {
        format!(
            "non-JSON response to {method}: {e}: {}",
            String::from_utf8_lossy(&body)
        )
    })
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn decode_chunked(mut body: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    // Bounded by the input length: every iteration consumes at least one byte.
    for _ in 0..=body.len() {
        let line_end = find_subslice(body, b"\r\n").ok_or("chunk size line")?;
        let size_text = String::from_utf8_lossy(&body[..line_end]);
        let size = usize::from_str_radix(size_text.trim().split(';').next().unwrap_or(""), 16)
            .map_err(|e| format!("chunk size {size_text:?}: {e}"))?;
        body = &body[line_end + 2..];
        if size == 0 {
            return Ok(out);
        }
        if body.len() < size + 2 {
            return Err("truncated chunk".to_string());
        }
        out.extend_from_slice(&body[..size]);
        body = &body[size + 2..];
    }
    Err("unterminated chunked body".to_string())
}
