//! MongoDB engine (P3). A concrete driver, not a `DbDriver`: that trait is
//! table/column/SQL/PK-shaped, Mongo is database -> collection -> document.
//!
//! Documents cross IPC as extended-JSON *text*. Going through `serde_json::Value`
//! (no `preserve_order` here) would sort keys and break compound/embedded `_id`
//! equality, and JS numbers would lose Int64 precision. Input is parsed straight
//! into `Bson` (typed, ordered); output is written by the small ordered `Ejson`.
//!
//! Safety: server-side JS operators are rejected in filters, the connection
//! string is never echoed or logged (no `Debug` on `MongoDriver`), and writes
//! are gated by the command layer's `require_editable`.
//!
//! ponytail: a fresh `Client` per call, like the SQL pool-per-call. Cache it in
//! `DbExplorerState` if remote TLS handshakes measurably hurt.

use futures_util::TryStreamExt;
use mongodb::bson::{doc, Bson, Document};
use mongodb::options::{ClientOptions, Credential, ServerAddress};
use mongodb::Client;
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::time::Duration;

use crate::domains::database_explorer::sql_server::CONNECT_TIMEOUT_SECS;
use crate::domains::database_explorer::sqlite::ROW_CAP;
use crate::domains::database_explorer::types::{MongoDoc, MongoPage};

/// Server-side cap on one find / count.
const QUERY_TIMEOUT: Duration = Duration::from_secs(30);
/// Operators that run JS on the server — never allowed in a filter.
const FORBIDDEN: [&str; 3] = ["$where", "$function", "$accumulator"];

pub struct MongoDriver {
    client: Client,
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

impl MongoDriver {
    /// `secret` is the stored password; if it is a `mongodb://` URI it is used
    /// as the whole connection string instead.
    pub async fn connect(
        host: &str,
        port: u16,
        user: &str,
        secret: &str,
        auth_db: &str,
    ) -> Result<Self, String> {
        let t = Duration::from_secs(CONNECT_TIMEOUT_SECS);
        if secret.starts_with("mongodb+srv://") {
            return Err("mongodb+srv:// is not supported — use a mongodb:// host list".into());
        }
        let mut o = if secret.starts_with("mongodb://") {
            // Generic message: the parse error can quote the URI (with its password).
            ClientOptions::parse(secret)
                .await
                .map_err(|_| "invalid MongoDB connection string".to_string())?
        } else {
            let mut o = ClientOptions::default();
            o.hosts = vec![ServerAddress::Tcp { host: host.to_string(), port: Some(port) }];
            o.direct_connection = Some(true);
            if !user.is_empty() {
                // `auth_db` is the vault entry's database; empty = server default (admin).
                let mut c = Credential::builder()
                    .username(user.to_string())
                    .password(secret.to_string())
                    .build();
                if !auth_db.is_empty() {
                    c.source = Some(auth_db.to_string());
                }
                o.credential = Some(c);
            }
            o
        };
        o.connect_timeout = Some(t);
        o.server_selection_timeout = Some(t);
        o.app_name = Some("portal-desktop".to_string());
        Ok(Self { client: Client::with_options(o).map_err(err)? })
    }

    fn coll(&self, db: &str, coll: &str) -> mongodb::Collection<Document> {
        self.client.database(db).collection(coll)
    }

    pub async fn list_databases(&self) -> Result<Vec<String>, String> {
        let mut v = self
            .client
            .list_database_names()
            .authorized_databases(true)
            .await
            .map_err(err)?;
        v.sort();
        Ok(v)
    }

    pub async fn list_collections(&self, db: &str) -> Result<Vec<String>, String> {
        let mut v = self.client.database(db).list_collection_names().await.map_err(err)?;
        v.sort();
        Ok(v)
    }

    pub async fn find(
        &self,
        db: &str,
        coll: &str,
        filter_text: &str,
        limit: usize,
        skip: u64,
    ) -> Result<MongoPage, String> {
        let filter = parse_filter(filter_text)?;
        let c = self.coll(db, coll);
        let cap = limit.clamp(1, ROW_CAP);
        let docs: Vec<Document> = c
            .find(filter.clone())
            .sort(doc! { "_id": 1 }) // stable paging
            .limit(cap as i64)
            .skip(skip)
            .max_time(QUERY_TIMEOUT)
            .await
            .map_err(err)?
            .try_collect()
            .await
            .map_err(err)?;
        // Empty filter: O(1) metadata count. Filtered: a real scan; if it fails or
        // times out the page still returns, with a lower-bound total (a full page
        // counts one extra so Next stays enabled).
        let total = if filter.is_empty() {
            c.estimated_document_count().max_time(QUERY_TIMEOUT).await.map_err(err)?
        } else {
            c.count_documents(filter).max_time(QUERY_TIMEOUT).await.unwrap_or(
                skip + docs.len() as u64 + u64::from(docs.len() == cap),
            )
        };
        Ok(MongoPage { docs: docs.into_iter().map(to_mongo_doc).collect(), total })
    }

    /// Returns the new document's `_id` as text.
    pub async fn insert(&self, db: &str, coll: &str, text: &str) -> Result<String, String> {
        let d = parse_doc(text)?;
        let r = self.coll(db, coll).insert_one(d).await.map_err(err)?;
        Ok(id_text(r.inserted_id))
    }

    pub async fn replace(&self, db: &str, coll: &str, id: &str, text: &str) -> Result<(), String> {
        let id = parse_id(id)?;
        let new = replacement(&id, text)?;
        let r = self.coll(db, coll).replace_one(id_filter(id), new).await.map_err(err)?;
        if r.matched_count == 0 {
            return Err("no document with that _id — nothing replaced".into());
        }
        Ok(())
    }

    pub async fn delete(&self, db: &str, coll: &str, id: &str) -> Result<(), String> {
        let id = parse_id(id)?;
        let r = self.coll(db, coll).delete_one(id_filter(id)).await.map_err(err)?;
        if r.deleted_count == 0 {
            return Err("no document with that _id — nothing deleted".into());
        }
        Ok(())
    }
}

/// Ordered extended-JSON tree (see module doc for why not `serde_json::Value`).
enum Ejson {
    Doc(Vec<(String, Ejson)>),
    Arr(Vec<Ejson>),
    Leaf(serde_json::Value),
}

impl Serialize for Ejson {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Ejson::Doc(v) => {
                let mut m = s.serialize_map(Some(v.len()))?;
                for (k, x) in v {
                    m.serialize_entry(k, x)?;
                }
                m.end()
            }
            Ejson::Arr(v) => s.collect_seq(v),
            Ejson::Leaf(v) => v.serialize(s),
        }
    }
}

/// `canonical` keeps numeric widths (`$numberLong`) — used for `_id`, which is
/// sent back verbatim. Document bodies use relaxed (readable) output.
fn ejson(b: Bson, canonical: bool) -> Ejson {
    match b {
        Bson::Document(d) => {
            Ejson::Doc(d.into_iter().map(|(k, v)| (k, ejson(v, canonical))).collect())
        }
        Bson::Array(a) => Ejson::Arr(a.into_iter().map(|v| ejson(v, canonical)).collect()),
        // Int widths survive the text round trip: plain JSON ints re-parse as
        // Int32 when they fit (and negatives as Int64), so only these are tagged.
        leaf @ (Bson::Int64(_) | Bson::Int32(..0)) => Ejson::Leaf(leaf.into_canonical_extjson()),
        leaf if canonical => Ejson::Leaf(leaf.into_canonical_extjson()),
        leaf => Ejson::Leaf(leaf.into_relaxed_extjson()),
    }
}

fn id_text(id: Bson) -> String {
    serde_json::to_string(&ejson(id, true)).unwrap_or_default()
}

fn to_mongo_doc(d: Document) -> MongoDoc {
    let id = d.get("_id").cloned().map(id_text);
    let json = serde_json::to_string_pretty(&ejson(Bson::Document(d), false)).unwrap_or_default();
    MongoDoc { id, json }
}

fn parse_doc(text: &str) -> Result<Document, String> {
    match serde_json::from_str::<Bson>(text).map_err(|e| format!("invalid JSON: {e}"))? {
        Bson::Document(d) => Ok(d),
        _ => Err("expected a JSON object".into()),
    }
}

/// `_id` is caller-supplied text, so it must be a plain value: null, regex and
/// operator documents (`{"$gt": ...}`) are rejected, and `id_filter` wraps it
/// in `$eq` so nothing that slips through can act as an operator.
fn parse_id(text: &str) -> Result<Bson, String> {
    let id = serde_json::from_str::<Bson>(text).map_err(|e| format!("invalid _id: {e}"))?;
    let bad = match &id {
        Bson::Null | Bson::RegularExpression(_) => true,
        Bson::Document(d) => d.keys().next().is_some_and(|k| k.starts_with('$')),
        _ => false,
    };
    if bad {
        return Err("invalid _id".into());
    }
    Ok(id)
}

fn id_filter(id: Bson) -> Document {
    doc! { "_id": { "$eq": id } }
}

fn parse_filter(text: &str) -> Result<Document, String> {
    if text.trim().is_empty() {
        return Ok(Document::new());
    }
    let d = parse_doc(text)?;
    check_filter(&Bson::Document(d.clone()))?;
    Ok(d)
}

/// Rejects server-side JS operators at any depth (`$expr`, `$or`, arrays...).
fn check_filter(b: &Bson) -> Result<(), String> {
    match b {
        Bson::Document(d) => d.iter().try_for_each(|(k, v)| {
            if FORBIDDEN.contains(&k.as_str()) {
                Err(format!("{k} is not allowed in a filter"))
            } else {
                check_filter(v)
            }
        }),
        Bson::Array(a) => a.iter().try_for_each(check_filter),
        _ => Ok(()),
    }
}

/// Ints compare by value so a displayed Int64 `_id` matches its re-parsed Int32.
fn same_id(a: &Bson, b: &Bson) -> bool {
    let n = |x: &Bson| match x {
        Bson::Int32(i) => Some(i64::from(*i)),
        Bson::Int64(i) => Some(*i),
        _ => None,
    };
    match (n(a), n(b)) {
        (Some(x), Some(y)) => x == y,
        _ => a == b,
    }
}

/// The replacement body for `replace_one({_id: orig}, ...)`: `_id` stripped
/// (it is immutable), and an `_id` that differs from the original is an error.
///
/// A hand-typed plain int still re-parses as Int32 when it fits;
/// `{"$numberLong": "5"}` forces the type.
fn replacement(orig: &Bson, text: &str) -> Result<Document, String> {
    if matches!(orig, Bson::Null) {
        return Err("document has a null _id — it cannot be replaced safely".into());
    }
    let mut d = parse_doc(text)?;
    if let Some(new) = d.remove("_id") {
        if !same_id(&new, orig) {
            return Err("_id cannot be changed — insert a new document instead".into());
        }
    }
    Ok(d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filter_ok(s: &str) -> bool {
        parse_filter(s).is_ok()
    }

    #[test]
    fn guard_rejects_js_at_any_depth() {
        assert!(!filter_ok(r#"{"$where": "1"}"#));
        assert!(!filter_ok(r#"{"$expr": {"$function": {"body": "x"}}}"#));
        assert!(!filter_ok(r#"{"$or": [{"a": 1}, {"$where": "1"}]}"#));
        assert!(!filter_ok(r#"{"a": {"$accumulator": {}}}"#));
        assert!(filter_ok(r#"{"a": {"$regex": "^x"}, "b": {"$gt": 3}}"#));
        assert!(filter_ok("  "));
    }

    #[test]
    fn ejson_round_trip_keeps_order_and_types() {
        let src = r#"{"z":1,"_id":{"$oid":"507f1f77bcf86cd799439011"},"a":{"y":2,"b":3},"d":{"$date":"2020-01-02T03:04:05Z"},"big":{"$numberLong":"9007199254740993"}}"#;
        let d = parse_doc(src).unwrap();
        let out = to_mongo_doc(d.clone());
        assert_eq!(out.id.as_deref(), Some(r#"{"$oid":"507f1f77bcf86cd799439011"}"#));
        let top: Vec<&str> = d.keys().map(String::as_str).collect();
        assert_eq!(top, ["z", "_id", "a", "d", "big"]);
        // pretty text lists top-level keys in the same order
        let pos: Vec<usize> = top.iter().map(|k| out.json.find(&format!("\"{k}\":")).unwrap()).collect();
        assert!(pos.windows(2).all(|w| w[0] < w[1]));
        assert!(out.json.contains("9007199254740993"));
        assert_eq!(parse_doc(&out.json).unwrap(), d);
        // canonical id keeps Int64
        assert_eq!(id_text(Bson::Int64(5)), r#"{"$numberLong":"5"}"#);
        assert_eq!(parse_id(&id_text(Bson::Int64(5))).unwrap(), Bson::Int64(5));
    }

    #[test]
    fn int_widths_and_compound_ids_survive_text_round_trip() {
        let d = doc! { "a": 5i64, "b": -3i32, "c": -3i64, "d": 7i32, "_id": doc! { "k": 5i64, "n": -1i32 } };
        let out = to_mongo_doc(d.clone());
        assert_eq!(parse_doc(&out.json).unwrap(), d);
        let id = parse_id(out.id.as_deref().unwrap()).unwrap();
        assert_eq!(id, d.get("_id").unwrap().clone());
        assert!(replacement(&id, &out.json).is_ok());
    }

    #[test]
    fn parse_id_rejects_null_and_operators() {
        assert!(parse_id("null").is_err());
        assert!(parse_id(r#"{"$gt": 1}"#).is_err());
        assert!(parse_id(r#"{"$gt": {"$minKey": 1}}"#).is_err());
        assert!(parse_id(r#"{"$regularExpression": {"pattern": ".*", "options": ""}}"#).is_err());
        assert!(parse_id(r#"{"$oid":"507f1f77bcf86cd799439011"}"#).is_ok());
        assert!(parse_id(r#"{"a": 1}"#).is_ok());
        assert_eq!(id_filter(Bson::Int32(1)), doc! { "_id": { "$eq": 1 } });
    }

    #[test]
    fn replacement_id_rules() {
        let oid = parse_id(r#"{"$oid":"507f1f77bcf86cd799439011"}"#).unwrap();
        let d = replacement(&oid, r#"{"_id":{"$oid":"507f1f77bcf86cd799439011"},"a":1}"#).unwrap();
        assert!(d.get("_id").is_none() && d.get_i32("a") == Ok(1));
        assert!(replacement(&oid, r#"{"_id":{"$oid":"507f1f77bcf86cd799439012"}}"#).is_err());
        assert!(replacement(&Bson::Null, r#"{"a":1}"#).is_err());
        assert!(replacement(&Bson::Int64(5), r#"{"_id":5}"#).is_ok());
        assert!(replacement(&oid, "[1]").is_err());
    }

    /// READ-ONLY against the owner's real local mongod: list + find only.
    #[tokio::test]
    #[ignore = "needs a mongod on 127.0.0.1:27017"]
    async fn live_list_and_find_read_only() {
        let d = MongoDriver::connect("127.0.0.1", 27017, "", "", "").await.unwrap();
        let dbs = d.list_databases().await.unwrap();
        assert!(!dbs.is_empty());
        for db in dbs.iter().filter(|n| n.as_str() != "local") {
            if let Some(c) = d.list_collections(db).await.unwrap().first() {
                let p = d.find(db, c, "", 2, 0).await.unwrap();
                assert!(p.docs.len() <= 2);
                return;
            }
        }
    }
}
