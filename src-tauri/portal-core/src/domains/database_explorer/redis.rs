//! Redis engine (P4). A concrete driver, not a `DbDriver`: keys / typed values /
//! TTLs, not tables and SQL.
//!
//! Safety: never `KEYS` (cursor `SCAN` only, bounded per call), values are
//! size-capped, the connection string / password is never echoed or logged (no
//! `Debug` on `RedisDriver`, connect errors are generic), and writes are gated
//! by the command layer's `require_editable`. A write is one command built by
//! `RedisWrite::args`, the same text the confirm dialog shows.
//!
//! ponytail: a fresh connection per call, like the SQL pool-per-call. Cache it
//! in `DbExplorerState` if remote handshakes measurably hurt.

use ::redis::aio::MultiplexedConnection;
use ::redis::{AsyncConnectionConfig, ConnectionAddr, IntoConnectionInfo};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use crate::domains::database_explorer::sql_server::CONNECT_TIMEOUT_SECS;
use crate::domains::database_explorer::sqlite::ROW_CAP;
use crate::domains::database_explorer::types::{RedisDb, RedisScan, RedisValue, RedisWrite};

const RESPONSE_TIMEOUT: Duration = Duration::from_secs(30);
/// Largest string body read (bytes).
const MAX_STRING: usize = 1 << 20;
/// Most SCAN rounds per call: bounds a sparse pattern on a huge db.
const MAX_ROUNDS: usize = 20;
/// Largest single list / set / hash / zset cell (bytes).
const MAX_CELL: usize = 64 << 10;

pub struct RedisDriver {
    con: MultiplexedConnection,
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn lossy(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// Cell text capped at `MAX_CELL` bytes; flips `cut` when it truncated.
fn cell(b: &[u8], cut: &mut bool) -> String {
    *cut |= b.len() > MAX_CELL;
    lossy(&b[..b.len().min(MAX_CELL)])
}

impl RedisDriver {
    /// `secret` is the stored password; if it is a `redis://` / `rediss://` URL
    /// it is used as the whole connection string instead. `db` is the logical
    /// database, selected on connect. `tls` is for host/port sources whose vault
    /// connection string was `rediss://`.
    pub async fn connect(
        host: &str,
        port: u16,
        user: &str,
        secret: &str,
        db: u32,
        tls: bool,
    ) -> Result<Self, String> {
        // Generic: parse errors can quote the URL (with its password).
        fn bad<E>(_: E) -> String {
            "invalid Redis connection string".to_string()
        }
        let info = if secret.starts_with("redis://") || secret.starts_with("rediss://") {
            secret.into_connection_info().map_err(bad)?
        } else {
            let mut r = ::redis::RedisConnectionInfo::default();
            if !user.is_empty() {
                r = r.set_username(user);
            }
            if !secret.is_empty() {
                r = r.set_password(secret);
            }
            let addr = if tls {
                ConnectionAddr::TcpTls {
                    host: host.to_string(),
                    port,
                    insecure: false,
                    tls_params: None,
                }
            } else {
                ConnectionAddr::Tcp(host.to_string(), port)
            };
            addr.into_connection_info()
                .map_err(bad)?
                .set_redis_settings(r)
        };
        let r = info.redis_settings().clone().set_db(i64::from(db));
        let info = info.set_redis_settings(r);
        let cfg = AsyncConnectionConfig::new()
            .set_connection_timeout(Some(Duration::from_secs(CONNECT_TIMEOUT_SECS)))
            .set_response_timeout(Some(RESPONSE_TIMEOUT));
        let con = ::redis::Client::open(info)
            .map_err(bad)?
            .get_multiplexed_async_connection_with_config(&cfg)
            .await
            // Not `e.to_string()`: for a URL source it may echo the URL.
            .map_err(|e| format!("cannot connect to Redis ({:?})", e.kind()))?;
        Ok(Self { con })
    }

    pub async fn list_databases(&self) -> Result<Vec<RedisDb>, String> {
        let mut c = self.con.clone();
        let info: String =
            ::redis::cmd("INFO").arg("keyspace").query_async(&mut c).await.map_err(err)?;
        // Managed Redis often disables CONFIG.
        let count = ::redis::cmd("CONFIG")
            .arg("GET")
            .arg("databases")
            .query_async::<Vec<String>>(&mut c)
            .await
            .ok()
            .and_then(|v| v.get(1)?.parse().ok());
        Ok(db_list(count, parse_keyspace(&info)))
    }

    pub async fn scan(&self, pattern: &str, cursor: &str) -> Result<RedisScan, String> {
        let mut cur: u64 = cursor.parse().map_err(|_| "invalid cursor".to_string())?;
        let pat = if pattern.is_empty() { "*" } else { pattern };
        let mut c = self.con.clone();
        let mut keys: Vec<String> = Vec::new();
        for _ in 0..MAX_ROUNDS {
            let (next, batch): (u64, Vec<Vec<u8>>) = ::redis::cmd("SCAN")
                .arg(cur)
                .arg("MATCH")
                .arg(pat)
                .arg("COUNT")
                .arg(500)
                .query_async(&mut c)
                .await
                .map_err(err)?;
            // ponytail: binary (non-UTF-8) keys are hidden; add a byte-safe key encoding if anyone has them.
            keys.extend(batch.into_iter().filter_map(|k| String::from_utf8(k).ok()));
            cur = next;
            if cur == 0 || keys.len() >= ROW_CAP {
                break;
            }
        }
        // SCAN may return a key more than once.
        keys.sort();
        keys.dedup();
        Ok(RedisScan { keys, cursor: cur.to_string() })
    }

    pub async fn get(&self, key: &str) -> Result<RedisValue, String> {
        let mut c = self.con.clone();
        let (kind, ttl): (String, i64) = ::redis::pipe()
            .cmd("TYPE")
            .arg(key)
            .cmd("TTL")
            .arg(key)
            .query_async(&mut c)
            .await
            .map_err(err)?;
        let mut v = RedisValue {
            kind: kind.clone(),
            ttl,
            len: 0,
            text: None,
            binary: false,
            items: Vec::new(),
            truncated: false,
        };
        let last = ROW_CAP - 1;
        let mut cut = false;
        match kind.as_str() {
            "none" => return Err("key not found — it may have expired or been deleted".into()),
            "string" => {
                let (len, body): (u64, Vec<u8>) = ::redis::pipe()
                    .cmd("STRLEN")
                    .arg(key)
                    .cmd("GETRANGE")
                    .arg(key)
                    .arg(0)
                    .arg(MAX_STRING - 1)
                    .query_async(&mut c)
                    .await
                    .map_err(err)?;
                v.len = len;
                v.truncated = len > MAX_STRING as u64;
                let (text, binary) = match String::from_utf8(body) {
                    Ok(s) => (s, false),
                    Err(e) => {
                        // Cut mid-char only by the size cap: keep the valid prefix.
                        let n = e.utf8_error().valid_up_to();
                        if v.truncated && e.utf8_error().error_len().is_none() {
                            (lossy(&e.as_bytes()[..n]), false)
                        } else {
                            (lossy(e.as_bytes()), true)
                        }
                    }
                };
                v.text = Some(text);
                v.binary = binary;
                return Ok(v);
            }
            "list" => {
                let (len, xs): (u64, Vec<Vec<u8>>) = ::redis::pipe()
                    .cmd("LLEN")
                    .arg(key)
                    .cmd("LRANGE")
                    .arg(key)
                    .arg(0)
                    .arg(last)
                    .query_async(&mut c)
                    .await
                    .map_err(err)?;
                v.len = len;
                v.items = xs.iter().map(|x| vec![cell(x, &mut cut)]).collect();
            }
            "zset" => {
                let (len, xs): (u64, Vec<Vec<u8>>) = ::redis::pipe()
                    .cmd("ZCARD")
                    .arg(key)
                    .cmd("ZRANGE")
                    .arg(key)
                    .arg(0)
                    .arg(last)
                    .arg("WITHSCORES")
                    .query_async(&mut c)
                    .await
                    .map_err(err)?;
                v.len = len;
                // Score stays the reply's text (f64 would lose precision).
                v.items =
                    xs.chunks(2).map(|p| p.iter().map(|x| cell(x, &mut cut)).collect()).collect();
            }
            "hash" | "set" => {
                let (lencmd, scan, width) =
                    if kind == "hash" { ("HLEN", "HSCAN", 2) } else { ("SCARD", "SSCAN", 1) };
                v.len = ::redis::cmd(lencmd).arg(key).query_async(&mut c).await.map_err(err)?;
                let mut cur = 0u64;
                let mut flat: Vec<Vec<u8>> = Vec::new();
                for _ in 0..MAX_ROUNDS {
                    let (next, batch): (u64, Vec<Vec<u8>>) = ::redis::cmd(scan)
                        .arg(key)
                        .arg(cur)
                        .arg("COUNT")
                        .arg(200)
                        .query_async(&mut c)
                        .await
                        .map_err(err)?;
                    flat.extend(batch);
                    cur = next;
                    if cur == 0 || flat.len() / width >= ROW_CAP {
                        break;
                    }
                }
                v.items = flat
                    .chunks(width)
                    .take(ROW_CAP)
                    .map(|p| p.iter().map(|x| cell(x, &mut cut)).collect())
                    .collect();
            }
            _ => {} // stream, ReJSON…: the UI says "not supported for viewing"
        }
        v.truncated = cut || v.len > v.items.len() as u64;
        Ok(v)
    }

    pub async fn write(&self, w: &RedisWrite) -> Result<(), String> {
        let args = w.args()?;
        let mut c = self.con.clone();
        if let RedisWrite::Set { key, create: false, .. } = w {
            // ponytail: TOCTOU between this check and the SET (XX still guards deletion).
            let t: String = ::redis::cmd("TYPE").arg(key).query_async(&mut c).await.map_err(err)?;
            if t == "none" {
                return Err("key no longer exists — reload".into());
            }
            if t != "string" {
                return Err("key is no longer a string — reload".into());
            }
        }
        let mut cmd = ::redis::cmd(&args[0]);
        for a in &args[1..] {
            cmd.arg(a);
        }
        match w {
            RedisWrite::Set { create, .. } => {
                // NX/XX reply nil when the condition fails.
                let r: Option<String> = cmd.query_async(&mut c).await.map_err(err)?;
                if r.is_none() {
                    return Err(if *create {
                        "key already exists".into()
                    } else {
                        "key no longer exists — reload".into()
                    });
                }
            }
            RedisWrite::Del { .. } => {
                let n: u64 = cmd.query_async(&mut c).await.map_err(err)?;
                if n == 0 {
                    return Err("no such key".into());
                }
            }
        }
        Ok(())
    }
}

impl RedisWrite {
    /// The exact command: NX when creating (never overwrite or change another
    /// key's type), XX when editing (never resurrect a deleted key), explicit EX
    /// instead of KEEPTTL so it works on Redis < 6 and the confirm text is what runs.
    pub fn args(&self) -> Result<Vec<String>, String> {
        match self {
            Self::Set { key, value, ttl, create } => {
                if key.is_empty() {
                    return Err("key is empty".into());
                }
                let mut a = vec![
                    "SET".to_string(),
                    key.clone(),
                    value.clone(),
                    (if *create { "NX" } else { "XX" }).to_string(),
                ];
                match ttl {
                    Some(0) => return Err("TTL must be at least 1 second".into()),
                    Some(t) => a.extend(["EX".to_string(), t.to_string()]),
                    None => {}
                }
                Ok(a)
            }
            Self::Del { key } if key.is_empty() => Err("key is empty".into()),
            Self::Del { key } => Ok(vec!["DEL".into(), key.clone()]),
        }
    }
}

/// redis-cli style: bare when plainly safe, else double-quoted and escaped.
pub fn command_text(args: &[String]) -> String {
    let safe = |a: &str| {
        !a.is_empty() && a.chars().all(|c| c.is_ascii_alphanumeric() || "_:.-/*".contains(c))
    };
    args.iter()
        .map(|a| {
            if safe(a) {
                return a.clone();
            }
            let mut s = String::from("\"");
            for c in a.chars() {
                match c {
                    '"' => s.push_str("\\\""),
                    '\\' => s.push_str("\\\\"),
                    '\n' => s.push_str("\\n"),
                    '\r' => s.push_str("\\r"),
                    '\t' => s.push_str("\\t"),
                    c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                        s.push_str(&format!("\\x{:02x}", c as u32))
                    }
                    c => s.push(c),
                }
            }
            s.push('"');
            s
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `INFO keyspace` lines: `db0:keys=5,expires=0,avg_ttl=0`.
fn parse_keyspace(info: &str) -> BTreeMap<u32, u64> {
    info.lines()
        .filter_map(|l| {
            let (db, rest) = l.trim().split_once(':')?;
            let idx = db.strip_prefix("db")?.parse().ok()?;
            let keys = rest.split(',').find_map(|kv| kv.strip_prefix("keys="))?.parse().ok()?;
            Some((idx, keys))
        })
        .collect()
}

/// Indices `0..min(count, 16)` plus every non-empty one, sorted, with counts.
fn db_list(count: Option<u32>, counts: BTreeMap<u32, u64>) -> Vec<RedisDb> {
    let mut idx: BTreeSet<u32> = (0..count.unwrap_or(1).min(16)).collect();
    idx.extend(counts.keys().copied());
    idx.into_iter()
        .map(|index| RedisDb { index, keys: counts.get(&index).copied().unwrap_or(0) })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(key: &str, value: &str, ttl: Option<u64>, create: bool) -> RedisWrite {
        RedisWrite::Set { key: key.into(), value: value.into(), ttl, create }
    }

    #[test]
    fn keyspace_and_db_list() {
        let m = parse_keyspace(
            "# Keyspace\r\ndb0:keys=5,expires=0,avg_ttl=0\r\ndb3:keys=2,expires=1,avg_ttl=9\r\n",
        );
        assert_eq!(m, BTreeMap::from([(0, 5), (3, 2)]));
        assert!(parse_keyspace("# Keyspace\r\n").is_empty());
        let l = db_list(Some(4), m);
        let got: Vec<_> = l.iter().map(|d| (d.index, d.keys)).collect();
        assert_eq!(got, [(0, 5), (1, 0), (2, 0), (3, 2)]);
        // CONFIG unavailable → db0 plus non-empty ones; count capped at 16
        assert_eq!(db_list(None, BTreeMap::from([(7, 1)])).len(), 2);
        assert_eq!(db_list(Some(1000), BTreeMap::new()).len(), 16);
    }

    #[test]
    fn cell_cap() {
        let mut cut = false;
        assert_eq!(cell(b"abc", &mut cut), "abc");
        assert!(!cut);
        assert_eq!(cell(&vec![b'a'; MAX_CELL + 1], &mut cut).len(), MAX_CELL);
        assert!(cut);
    }

    #[test]
    fn write_args_and_guards() {
        assert_eq!(set("k", "v", None, true).args().unwrap(), ["SET", "k", "v", "NX"]);
        assert_eq!(
            set("k", "v", Some(60), false).args().unwrap(),
            ["SET", "k", "v", "XX", "EX", "60"]
        );
        assert!(set("k", "v", Some(0), true).args().is_err());
        assert!(set("", "v", None, true).args().is_err());
        assert_eq!(RedisWrite::Del { key: "k".into() }.args().unwrap(), ["DEL", "k"]);
        assert!(RedisWrite::Del { key: String::new() }.args().is_err());
    }

    #[test]
    fn command_text_quotes() {
        let t = |a: &[&str]| command_text(&a.iter().map(|s| s.to_string()).collect::<Vec<_>>());
        assert_eq!(t(&["SET", "user:1", "x"]), "SET user:1 x");
        assert_eq!(t(&["SET", "k", ""]), r#"SET k """#);
        assert_eq!(t(&["SET", "a b", "q\"\\\n\r\t\u{1}"]), r#"SET "a b" "q\"\\\n\r\t\x01""#);
        assert_eq!(t(&["SET", "é"]), "SET \"é\"");
    }

    #[test]
    fn write_deserializes_from_wire_tags() {
        let w: RedisWrite =
            serde_json::from_str(r#"{"op":"set","key":"k","value":"v","ttl":null,"create":true}"#)
                .unwrap();
        assert!(matches!(w, RedisWrite::Set { create: true, .. }));
        assert!(serde_json::from_str::<RedisWrite>(r#"{"op":"del","key":"k"}"#).is_ok());
    }

    /// READ-ONLY against a local Redis: list dbs, scan db0, get the first key.
    #[tokio::test]
    #[ignore = "needs redis on 127.0.0.1:6379"]
    async fn live_list_scan_get_read_only() {
        let d = RedisDriver::connect("127.0.0.1", 6379, "", "", 0, false).await.unwrap();
        assert!(!d.list_databases().await.unwrap().is_empty());
        let s = d.scan("", "0").await.unwrap();
        if let Some(k) = s.keys.first() {
            d.get(k).await.unwrap();
        }
    }
}
