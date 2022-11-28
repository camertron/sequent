use sqlite::Value;

pub mod build_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
 }

 #[derive(Debug)]
 pub struct QueryResultHeader {
    pub row_count: usize,
    pub column_count: usize,
    pub columns: Vec<String>,
}

#[derive(Debug)]
pub struct QueryResult {
    pub header: QueryResultHeader,
    pub rows: Vec<Vec<Value>>,
}

pub const DEFAULT_PORT: u16 = 9087;
