use clap::{ArgAction, Parser};
use log::{LevelFilter, debug, info, error};
use std::collections::LinkedList;
use std::process::exit;
use sequent::{DEFAULT_PORT, ValueWrapper, QueryResultRow, QueryResult};
use sequent::build_info::{PKG_VERSION};
use sqlite;
use sqlite::Value;
use zmq;

#[derive(Parser, Debug)]
#[command(author="Cameron C. Dutro")]
#[command(version="1.0.0")]
#[command(about="Provide access a SQLite database over TCP.")]
struct CLI {
    #[arg(long, short, value_name="PATH", help="The SQLite databse file to use.")]
    file: String,

    #[arg(long, short, value_name="ADDR", help="The address to bind to.")]
    #[arg(default_value_t=String::from(format!("127.0.0.1:{}", DEFAULT_PORT)))]
    bind: String,

    #[arg(long=None, short='v', help="Enable verbose logging.", action = ArgAction::Count)]
    log_level: Option<u8>
}

fn main() {
    let options = CLI::parse();

    let level_filter = match options.log_level {
        None => LevelFilter::Info,
        Some(lvl) => match lvl {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            _ => LevelFilter::Trace
        }
    };

    simplelog::TermLogger::init(
        level_filter,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto
    ).unwrap();

    let db = sqlite::open(options.file).unwrap();
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::REP).unwrap();
    let addr = format!("tcp://{}", options.bind);

    socket.bind(addr.as_str()).unwrap_or_else(|e| {
        error!("Could not bind to {}: {}", addr, e);
        exit(1);
    });

    info!("Sequent server v{} listening on {}", PKG_VERSION, addr);

    let mut msg = zmq::Message::new();

    loop {
        debug!("Waiting for message");

        if let Err(e) = socket.recv(&mut msg, 0) {
            error!("Error when attempting to receive message: {}", e);
            continue;
        }

        debug!("Received message, {} bytes", msg.len());

        let query = match msg.as_str() {
            Some(q) => q,
            None => {
                error!("Received query that was not valid utf-8 text");
                continue;
            }
        };

        let statement = match db.prepare(query) {
            Ok(s) => s,
            Err(e) => {
                error!("Error preparing SQL statement: {}", e);
                continue;
            }
        };

        info!("QUERY: {}", query);

        let mut cursor = statement.into_iter();
        let mut result_rows: LinkedList<QueryResultRow> = LinkedList::new();

        while let Some(Ok(row)) = cursor.next() {
            let values: Vec<Value> = row.into();
            let mut wrapped_values: Vec<ValueWrapper> = Vec::with_capacity(values.len());

            for value in values {
                wrapped_values.push(ValueWrapper(value));
            }

            result_rows.push_back(wrapped_values);
        }

        let result = QueryResult {
            columns: cursor.column_names().to_vec(),
            column_count: cursor.column_count(),
            rows: result_rows.into_iter().collect()
        };

        let buf = rmp_serde::to_vec(&result).unwrap();

        match socket.send(buf, 0) {
            Ok(_) => info!("Sent {} row(s) to client", result.rows.len()),
            Err(e) => error!("Error sending rows to client: {}", e)
        }
    }
}
