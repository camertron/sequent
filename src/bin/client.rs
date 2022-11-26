extern crate sequent;

use clap::{Parser};
use sequent::{DEFAULT_PORT, QueryResult};
use sqlite::Value;
use std::ops::Deref;
use zmq;

#[derive(Parser)]
#[command(author="Cameron C. Dutro")]
#[command(version="1.0.0")]
#[command(about="Make requests to a Sequent server.")]
struct CLI {
    #[arg(long, short='H', help="The host and port of the Sequent server.")]
    #[arg(default_value_t=String::from(format!("127.0.0.1:{}", DEFAULT_PORT)))]
    host: String,

    #[arg(long, short, help="The SQL query to execute.")]
    query: String
}

fn main() {
    let options = CLI::parse();

    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::REQ).unwrap();

    socket.connect(format!("tcp://{}", options.host).as_str()).unwrap();
    socket.send(&options.query, 0).unwrap();

    let message = socket.recv_msg(0).unwrap();
    let message_bytes = message.deref();
    let results = rmp_serde::from_slice::<QueryResult>(&message_bytes).unwrap();
    let mut longest = vec![0; results.column_count];

    // for row in results.rows {
    let row_strings = results.rows.iter().map(|row| {
        row.iter().enumerate().map(|(column_index, value)| {
            let value_str = match &value.0 {
                Value::String(str) => str.clone(),
                Value::Binary(bytes) => String::from_utf8(bytes.to_vec()).unwrap(),
                Value::Float(f) => format!("{}", f),
                Value::Integer(i) => format!("{}", i),
                Value::Null => "NULL".into(),
            };

            if value_str.len() > longest[column_index] {
                longest[column_index] = value_str.len();
            }

            value_str
        }).collect::<Vec<String>>()
    }).collect::<Vec<Vec<String>>>();

    for row in row_strings {
        for (column_index, s) in row.iter().enumerate() {
            if column_index > 0 {
                print!("|");
            }

            let width = longest[column_index];
            print!("{:width$}", s, width = width);
        }

        print!("{}", "\n");
    }
}
