use bytes::{BytesMut, BufMut};
use clap::{ArgAction, Parser};
use log::{LevelFilter, debug, info, error};
use std::process::exit;
use sequent::DEFAULT_PORT;
use sequent::build_info::PKG_VERSION;
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
        let mut row_results = BytesMut::new();
        let mut row_count: u64 = 0;

        while let Some(Ok(row)) = cursor.next() {
            let values: Vec<Value> = row.into();

            for value in values {
                match value {
                    Value::Null => {
                        row_results.put_u8(0);
                    }

                    Value::Integer(i) => {
                        row_results.put_u8(1);
                        row_results.put_i64(i);
                    }

                    Value::Float(f) => {
                        row_results.put_u8(2);
                        row_results.put_f64(f);
                    }

                    Value::String(str) => {
                        row_results.put_u8(3);
                        let bytes = str.as_bytes();
                        row_results.put_u64(bytes.len() as u64);
                        row_results.put_slice(bytes);
                    }

                    Value::Binary(bytes) => {
                        row_results.put_u8(4);
                        row_results.put_u64(bytes.len() as u64);
                        row_results.put_slice(bytes.as_slice());
                    }
                }
            }

            row_count += 1;
        }

        let column_names = cursor.column_names();
        let mut header = BytesMut::new();

        header.put_slice(&b"SQNT"[..]);
        header.put_u64(row_count);
        header.put_u64(column_names.len() as u64);

        for column_name in column_names {
            header.put_u64(column_name.len() as u64);
            header.put_slice(column_name.as_bytes());
        }

        match socket.send_multipart([header.to_vec(), row_results.to_vec()], 0) {
            Ok(_) => info!("Sent {} row(s) to client", row_count),
            Err(e) => error!("Error sending rows to client: {}", e)
        }
    }
}
