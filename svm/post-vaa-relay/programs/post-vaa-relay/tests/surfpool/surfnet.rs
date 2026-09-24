//! Surfpool subprocess guard and the RPC methods the tests use, including
//! the `surfnet_*` cheatcodes that seed state.

use std::{
    io::{BufRead, BufReader, Read},
    net::TcpListener,
    path::PathBuf,
    process::{Child, Command, Stdio},
    str::FromStr,
    thread,
    time::{Duration, Instant},
};

use base64::Engine;
use serde_json::{json, Value};
use solana_hash::Hash;
use solana_pubkey::Pubkey;

use super::rpc::{hex_encode, rpc_call, try_rpc};

/// SIMD-0385 feature gate.
pub const ENABLE_TX_V1_FEATURE: &str = "txv1aq4pp281K9um3tnPgkfX8UqtFT6wcVW3hNezGLL";
pub const MIN_SURFPOOL_VERSION: (u64, u64, u64) = (1, 6, 0);

const SURFPOOL_BOOT_TIMEOUT: Duration = Duration::from_secs(45);
const CONFIRM_TIMEOUT: Duration = Duration::from_secs(30);
const POLL_INTERVAL: Duration = Duration::from_millis(200);

/// `Drop` kills the child even on panic.
pub struct Surfpool {
    child: Child,
    pub rpc_url: String,
    _stdout_pump: Option<thread::JoinHandle<()>>,
    _stderr_pump: Option<thread::JoinHandle<()>>,
}

impl Drop for Surfpool {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    listener.local_addr().expect("local_addr").port()
}

fn surfpool_binary() -> PathBuf {
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join("surfpool");
            if candidate.is_file() {
                return candidate;
            }
        }
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let candidate = PathBuf::from(home).join(".local/bin/surfpool");
    assert!(
        candidate.exists(),
        "surfpool not found on PATH or at ~/.local/bin/surfpool"
    );
    candidate
}

fn pump(reader: impl Read + Send + 'static, tag: &'static str) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for line in BufReader::new(reader).lines().map_while(Result::ok) {
            eprintln!("[surfpool {tag}] {line}");
        }
    })
}

/// Boot an offline surfnet with the v1 feature gate active and wait for `getHealth`.
pub fn start_surfpool() -> Surfpool {
    let bin = surfpool_binary();
    let rpc_port = free_port();
    let scratch = std::env::temp_dir().join(format!("post-vaa-relay-surfpool-{rpc_port}"));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).expect("create scratch dir");

    let mut child = Command::new(&bin)
        .args([
            "start",
            "--offline",
            "--no-tui",
            "--no-studio",
            "--no-deploy",
            "-y",
        ])
        .args(["--port", &rpc_port.to_string()])
        .args(["--ws-port", &free_port().to_string()])
        .args(["--studio-port", &free_port().to_string()])
        .args(["--feature", ENABLE_TX_V1_FEATURE])
        .args(["--log-level", "warn"])
        .current_dir(&scratch)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn surfpool");
    let stdout = child.stdout.take().map(|s| pump(s, "stdout"));
    let stderr = child.stderr.take().map(|s| pump(s, "stderr"));

    let surfpool = Surfpool {
        child,
        rpc_url: format!("http://127.0.0.1:{rpc_port}"),
        _stdout_pump: stdout,
        _stderr_pump: stderr,
    };

    let deadline = Instant::now() + SURFPOOL_BOOT_TIMEOUT;
    let mut last = String::new();
    while Instant::now() < deadline {
        match try_rpc(&surfpool.rpc_url, "getHealth", json!([])) {
            Ok(v) if v.get("result").and_then(Value::as_str) == Some("ok") => {
                surfpool.require_min_version();
                return surfpool;
            }
            Ok(v) => last = v.to_string(),
            Err(e) => last = e,
        }
        thread::sleep(POLL_INTERVAL);
    }
    panic!("surfpool did not become healthy within {SURFPOOL_BOOT_TIMEOUT:?}: {last}");
}

impl Surfpool {
    pub fn rpc(&self, method: &str, params: Value) -> Value {
        rpc_call(&self.rpc_url, method, params)
    }

    fn require_min_version(&self) {
        let version = self.rpc("getVersion", json!([]));
        let text = version["result"]["surfnet-version"]
            .as_str()
            .unwrap_or_else(|| panic!("getVersion without surfnet-version: {version}"))
            .to_string();
        let parsed = parse_semver(&text).unwrap_or_else(|| panic!("surfpool version {text}"));
        assert!(
            parsed >= MIN_SURFPOOL_VERSION,
            "surfpool {text} is older than the required 1.6.0 (full transaction v1 support)"
        );
        eprintln!(
            "[surfpool] version {text}, solana-core {}",
            version["result"]["solana-core"]
        );
    }

    pub fn airdrop(&self, to: &Pubkey, lamports: u64) {
        let resp = self.rpc("requestAirdrop", json!([to.to_string(), lamports]));
        assert!(resp.get("error").is_none(), "requestAirdrop failed: {resp}");
        let deadline = Instant::now() + CONFIRM_TIMEOUT;
        while Instant::now() < deadline {
            let balance = self.rpc("getBalance", json!([to.to_string()]));
            if balance["result"]["value"].as_u64().unwrap_or(0) >= lamports {
                return;
            }
            thread::sleep(POLL_INTERVAL);
        }
        panic!("airdrop to {to} did not land");
    }

    pub fn latest_blockhash(&self) -> Hash {
        let resp = self.rpc("getLatestBlockhash", json!([{"commitment": "confirmed"}]));
        let text = resp["result"]["value"]["blockhash"]
            .as_str()
            .unwrap_or_else(|| panic!("getLatestBlockhash: {resp}"));
        Hash::from_str(text).expect("blockhash base58")
    }

    /// Send with preflight (load-stage failures return as an RPC error), then
    /// poll until the signature has a status.
    pub fn send_and_confirm(&self, tx_bytes: &[u8]) -> String {
        let encoded = base64::engine::general_purpose::STANDARD.encode(tx_bytes);
        let resp = self.rpc(
            "sendTransaction",
            json!([encoded, {"encoding": "base64", "skipPreflight": false, "preflightCommitment": "confirmed"}]),
        );
        let signature = resp["result"]
            .as_str()
            .unwrap_or_else(|| panic!("sendTransaction failed: {resp}"))
            .to_string();
        let deadline = Instant::now() + CONFIRM_TIMEOUT;
        while Instant::now() < deadline {
            let status = self.rpc("getSignatureStatuses", json!([[signature]]));
            let entry = &status["result"]["value"][0];
            if !entry.is_null() {
                let level = entry["confirmationStatus"].as_str().unwrap_or("");
                if !entry["err"].is_null() || level == "confirmed" || level == "finalized" {
                    return signature;
                }
            }
            thread::sleep(POLL_INTERVAL);
        }
        panic!("transaction {signature} not confirmed within {CONFIRM_TIMEOUT:?}");
    }

    /// `getTransaction` with `maxSupportedTransactionVersion: 1`.
    pub fn get_transaction(&self, signature: &str) -> Value {
        let resp = self.rpc(
            "getTransaction",
            json!([signature, {"encoding": "json", "commitment": "confirmed", "maxSupportedTransactionVersion": 1}]),
        );
        assert!(
            !resp["result"].is_null(),
            "getTransaction {signature}: {resp}"
        );
        resp["result"].clone()
    }

    /// `simulateTransaction` with signature verification. Returns `result.value`.
    pub fn simulate(&self, tx_bytes: &[u8]) -> Value {
        let encoded = base64::engine::general_purpose::STANDARD.encode(tx_bytes);
        let resp = self.rpc(
            "simulateTransaction",
            json!([encoded, {"encoding": "base64", "sigVerify": true, "commitment": "confirmed"}]),
        );
        assert!(
            resp.get("error").is_none(),
            "simulateTransaction failed: {resp}"
        );
        resp["result"]["value"].clone()
    }

    pub fn get_account(&self, pubkey: &Pubkey) -> Option<Account> {
        let resp = self.rpc(
            "getAccountInfo",
            json!([pubkey.to_string(), {"encoding": "base64", "commitment": "confirmed"}]),
        );
        let value = &resp["result"]["value"];
        if value.is_null() {
            return None;
        }
        Some(Account {
            lamports: value["lamports"].as_u64().expect("lamports"),
            owner: Pubkey::from_str(value["owner"].as_str().expect("owner")).expect("owner key"),
            executable: value["executable"].as_bool().expect("executable"),
            data: base64::engine::general_purpose::STANDARD
                .decode(value["data"][0].as_str().expect("data"))
                .expect("account data base64"),
        })
    }

    pub fn set_account(&self, pubkey: &Pubkey, account: &Account) {
        let resp = self.rpc(
            "surfnet_setAccount",
            json!([pubkey.to_string(), {
                "lamports": account.lamports,
                "owner": account.owner.to_string(),
                "executable": account.executable,
                "rent_epoch": 0,
                "data": hex_encode(&account.data),
            }]),
        );
        assert!(
            resp.get("error").is_none(),
            "surfnet_setAccount failed: {resp}"
        );
        let readback = self.get_account(pubkey).expect("account after setAccount");
        assert_eq!(
            readback.data, account.data,
            "setAccount data readback for {pubkey}"
        );
        assert_eq!(
            readback.owner, account.owner,
            "setAccount owner readback for {pubkey}"
        );
    }

    pub fn write_program(&self, program_id: &Pubkey, elf: &[u8]) {
        let resp = self.rpc(
            "surfnet_writeProgram",
            json!([program_id.to_string(), hex_encode(elf), 0]),
        );
        assert!(
            resp.get("error").is_none(),
            "surfnet_writeProgram failed: {resp}"
        );
        let account = self
            .get_account(program_id)
            .expect("program account after writeProgram");
        assert!(
            account.executable,
            "{program_id} is executable after writeProgram"
        );
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Account {
    pub lamports: u64,
    pub owner: Pubkey,
    pub executable: bool,
    pub data: Vec<u8>,
}

fn parse_semver(text: &str) -> Option<(u64, u64, u64)> {
    let mut parts = text.trim().split('.').map(|p| p.parse::<u64>().ok());
    Some((parts.next()??, parts.next()??, parts.next()??))
}
