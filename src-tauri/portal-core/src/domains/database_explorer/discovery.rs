//! Connection discovery: merge three sources into the explorer's source list.
//!
//! 1. Vault credentials of type DATABASE (host/port/db/user/engine from the
//!    plaintext metadata; the password stays `Vault(id)` and is decrypted only at
//!    connect time, so listing works even while the vault is locked).
//! 2. Running SDK-managed DB services (Postgres/MySQL).
//! 3. Running Docker containers whose image is Postgres/MySQL, at their actual
//!    mapped host port.
//!
//! Every source is best-effort: a failure in one path (vault DB error, Docker not
//! installed, …) is skipped, never fatal — `db_list_sources` still returns the rest.
//!
//! ponytail: SDK/Docker sources assume default local credentials (pg: postgres,
//! mysql: root, empty password); if that is wrong the connect error surfaces.

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use serde_json::Value;

use crate::domains::credentials::services::{CredentialService, VaultSession};
use crate::domains::database_explorer::state::{PasswordRef, ServerConn, Source};
use crate::domains::deployments::services::docker_service::DockerService;
use crate::domains::sdk::configs::SDKCategory;
use crate::domains::sdk::services::language_config_service::SDKConfigService;

/// Merge all discovery sources into `(id, Source)` pairs for the state registry.
pub async fn discover(db: DatabaseConnection, session: Arc<VaultSession>) -> Vec<(String, Source)> {
    let mut out = Vec::new();
    out.extend(from_vault(db, session).await);
    out.extend(from_sdk().await);
    out.extend(from_docker().await);
    out
}

async fn from_vault(db: DatabaseConnection, session: Arc<VaultSession>) -> Vec<(String, Source)> {
    let service = CredentialService::new(db, session);
    let creds = match service.get_credentials().await {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    creds
        .into_iter()
        .filter(|c| c.credential_type.eq_ignore_ascii_case("database"))
        .filter_map(|c| {
            let meta: Value = serde_json::from_str(&c.metadata).unwrap_or(Value::Null);
            let host = meta.get("host").and_then(Value::as_str)?.to_string();
            if host.is_empty() {
                return None;
            }
            let engine = infer_engine(&meta);
            let port = meta
                .get("port")
                .and_then(as_u16)
                .unwrap_or(default_port(&engine));
            let database = meta
                .get("database")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let user = meta
                .get("username")
                .and_then(Value::as_str)
                .unwrap_or(default_user(&engine))
                .to_string();
            Some((
                c.id.clone(),
                Source::Server(ServerConn {
                    engine: engine.clone(),
                    host: host.clone(),
                    port,
                    user,
                    database,
                    password: PasswordRef::Vault(c.id),
                    label: format!("{} ({engine})", c.name),
                    detail: format!("{host}:{port}"),
                }),
            ))
        })
        .collect()
}

async fn from_sdk() -> Vec<(String, Source)> {
    let configs = SDKConfigService::get_sdks_by_category(SDKCategory::Database)
        .await
        .unwrap_or_default();
    configs
        .into_iter()
        .filter(|c| c.service_running == Some(true))
        .filter_map(|c| {
            let port = c.service_port?;
            let engine = match c.id.as_str() {
                "postgresql" => "postgres",
                "mysql" => "mysql",
                _ => return None,
            };
            let id = format!("sdk:{}", c.id);
            let label = format!("{} (SDK service)", c.name);
            Some((id, local_server(engine, port, label, format!("127.0.0.1:{port}"))))
        })
        .collect()
}

async fn from_docker() -> Vec<(String, Source)> {
    let containers = DockerService::new().list_containers().await.unwrap_or_default();
    containers
        .into_iter()
        .filter(|c| c.status.starts_with("Up"))
        .filter_map(|c| {
            let image = c.image.to_lowercase();
            let (engine, container_port) = if image.contains("postgres") {
                ("postgres", 5432)
            } else if image.contains("mysql") || image.contains("maria") {
                ("mysql", 3306)
            } else {
                return None;
            };
            let host_port = mapped_host_port(&c.ports, container_port)?;
            let id = format!("docker:{}", c.id);
            let label = format!("{} (Docker)", c.name);
            Some((
                id,
                local_server(engine, host_port, label, format!("127.0.0.1:{host_port} · {}", c.image)),
            ))
        })
        .collect()
}

/// A local server source with default credentials (SDK / Docker).
fn local_server(engine: &str, port: u16, label: String, detail: String) -> Source {
    let (user, database) = match engine {
        "mysql" => ("root".to_string(), String::new()),
        _ => ("postgres".to_string(), "postgres".to_string()),
    };
    Source::Server(ServerConn {
        engine: engine.to_string(),
        host: "127.0.0.1".to_string(),
        port,
        user,
        database,
        password: PasswordRef::Plain(String::new()),
        label,
        detail,
    })
}

fn infer_engine(meta: &Value) -> String {
    // Explicit hint on the credential wins.
    for key in ["engine", "driver", "provider", "type", "dialect"] {
        if let Some(s) = meta.get(key).and_then(Value::as_str) {
            let s = s.to_lowercase();
            if s.contains("mysql") || s.contains("maria") {
                return "mysql".to_string();
            }
            if s.contains("postgres") || s == "pg" {
                return "postgres".to_string();
            }
        }
    }
    // Then the connection string scheme.
    if let Some(cs) = meta.get("connectionString").and_then(Value::as_str) {
        let cs = cs.to_lowercase();
        if cs.starts_with("mysql") {
            return "mysql".to_string();
        }
        if cs.starts_with("postgres") {
            return "postgres".to_string();
        }
    }
    // Finally the port.
    match meta.get("port").and_then(as_u16) {
        Some(3306) | Some(33060) => "mysql".to_string(),
        _ => "postgres".to_string(),
    }
}

fn default_port(engine: &str) -> u16 {
    if engine == "mysql" {
        3306
    } else {
        5432
    }
}

fn default_user(engine: &str) -> &'static str {
    if engine == "mysql" {
        "root"
    } else {
        "postgres"
    }
}

/// JSON number or string → u16.
fn as_u16(v: &Value) -> Option<u16> {
    v.as_u64()
        .and_then(|n| u16::try_from(n).ok())
        .or_else(|| v.as_str().and_then(|s| s.trim().parse().ok()))
}

/// Parse the mapped host port for `container_port` out of `docker ps` port
/// strings like `0.0.0.0:5432->5432/tcp` or `:::49153->3306/tcp`.
fn mapped_host_port(ports: &[String], container_port: u16) -> Option<u16> {
    let needle = format!("->{container_port}/");
    for p in ports {
        if let Some(idx) = p.find(&needle) {
            let left = &p[..idx]; // e.g. "0.0.0.0:5432" or ":::49153"
            if let Some(hp) = left.rsplit(':').next() {
                if let Ok(n) = hp.trim().parse::<u16>() {
                    return Some(n);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_mapped_host_port() {
        let ports = vec!["0.0.0.0:5432->5432/tcp".to_string()];
        assert_eq!(mapped_host_port(&ports, 5432), Some(5432));

        // Remapped host port differs from the container port.
        let ports = vec!["0.0.0.0:49153->3306/tcp".to_string(), ":::49153->3306/tcp".to_string()];
        assert_eq!(mapped_host_port(&ports, 3306), Some(49153));

        // Container port not published → None.
        let ports = vec!["5432/tcp".to_string()];
        assert_eq!(mapped_host_port(&ports, 5432), None);
        assert_eq!(mapped_host_port(&[], 5432), None);
    }

    #[test]
    fn infers_engine_from_hint_then_scheme_then_port() {
        assert_eq!(infer_engine(&json!({ "engine": "MySQL" })), "mysql");
        assert_eq!(infer_engine(&json!({ "provider": "postgresql" })), "postgres");
        assert_eq!(
            infer_engine(&json!({ "connectionString": "mysql://u:p@h/db" })),
            "mysql"
        );
        assert_eq!(infer_engine(&json!({ "port": 3306 })), "mysql");
        assert_eq!(infer_engine(&json!({ "port": 5432 })), "postgres");
        // Nothing to go on → default postgres.
        assert_eq!(infer_engine(&json!({})), "postgres");
    }

    #[test]
    fn reads_port_from_number_or_string() {
        assert_eq!(as_u16(&json!(5432)), Some(5432));
        assert_eq!(as_u16(&json!("3306")), Some(3306));
        assert_eq!(as_u16(&json!(70000)), None);
    }
}
