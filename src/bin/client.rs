extern crate sequent;

use bytes::{BytesMut, Buf};
use clap::{Parser};
use sequent::{DEFAULT_PORT, QueryResult, QueryResultHeader};
use sqlite::Value;
use std::process::exit;

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

    let message_parts = socket.recv_multipart(0).unwrap();
    let header = deserialize_header(&message_parts[0]);

    let rows = if message_parts.len() > 1 {
        deserialize_rows(&message_parts[1], &header)
    } else {
        vec![]
    };

    let result = QueryResult { header: header, rows: rows };

    print_result(&result);
}

fn deserialize_header(bytes: &Vec<u8>) -> QueryResultHeader {
    let mut message = BytesMut::from(bytes.as_slice());

    match message.get(0..4) {
        Some(b"SQNT") => message.advance(4),
        _ => {
            println!("Malformed response from server");
            exit(1);
        }
    }

    let row_count = message.get_u64();
    let column_count = message.get_u64();
    let mut column_names = Vec::with_capacity(column_count as usize);

    for _ in 0..column_count {
        let len = message.get_u64() as usize;
        let name = message.get(0..len).unwrap();
        column_names.push(String::from_utf8(name.to_vec()).unwrap());
        message.advance(len);
    }

    QueryResultHeader {
        row_count: row_count as usize,
        column_count: column_count as usize,
        columns: column_names
    }
}

fn deserialize_rows(bytes: &Vec<u8>, header: &QueryResultHeader) -> Vec<Vec<Value>> {
    let mut message = BytesMut::from(bytes.as_slice());
    let mut rows: Vec<Vec<Value>> = Vec::with_capacity(header.row_count);

    for _ in 0..header.row_count {
        let mut row: Vec<Value> = Vec::with_capacity(header.column_count);

        for _ in 0..header.column_count {
            let val = match message.get_u8() {
                0 => Value::Null,
                1 => Value::Integer(message.get_i64()),
                2 => Value::Float(message.get_f64()),
                3 => {
                    let len = message.get_u64() as usize;
                    let str = message.get(0..len).unwrap();
                    let val = Value::String(String::from_utf8(str.to_vec()).unwrap());
                    message.advance(len);
                    val
                }
                4 => {
                    let len = message.get_u64() as usize;
                    let bytes = message.get(0..len).unwrap();
                    let val = Value::Binary(bytes.to_vec());
                    message.advance(len);
                    val
                }
                _ => {
                    println!("Malformed response from server");
                    exit(1);
                }
            };

            row.push(val);
        }

        rows.push(row);
    }

    rows
}

fn print_result(result: &QueryResult) {
    if result.header.row_count == 0 {
        println!("(empty result set)");
        return;
    }

    let mut longest = vec![0; result.header.column_count];

    let row_strings = result.rows.iter().map(|row| {
        row.iter().enumerate().map(|(column_index, value)| {
            let value_str = match &value {
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
