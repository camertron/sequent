use serde::{Serialize, Deserialize};
use sqlite::Value;

pub mod build_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
 }

#[derive(Serialize, Deserialize)]
#[serde(remote = "Value")]
pub enum ValueDef {
    Binary(Vec<u8>),
    Float(f64),
    Integer(i64),
    String(String),
    Null,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ValueWrapper(#[serde(with = "ValueDef")] pub Value);

pub type QueryResultRow = Vec<ValueWrapper>;
pub type QueryResultRows = Vec<QueryResultRow>;

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub column_count: usize,
    pub rows: QueryResultRows,
}

pub const DEFAULT_PORT: u16 = 9087;
