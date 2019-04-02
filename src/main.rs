#[macro_use]
extern crate enum_display_derive;
extern crate chrono;
extern crate csv;
extern crate env_logger;
extern crate ws;
extern crate uuid;

mod wire;
use wire::bitmex;

use std::sync::mpsc;
use std::thread;

struct Client<T> {
    out: ws::Sender,
    tx: mpsc::Sender<T>,
}

struct CsvSink<W: std::io::Write> {
    writer: csv::Writer<W>,
}

trait StreamProcessor<T: serde::Serialize> {
    fn process(&mut self, msg: &T);
}

impl<W: std::io::Write> CsvSink<W> {
    fn new(write: W) -> CsvSink<W> {
        let writer = csv::WriterBuilder::new()
            .delimiter(b'|')
            .has_headers(false)
            .flexible(true)
            .from_writer(write);
        CsvSink { writer: writer }
    }

    fn serialize<T: serde::Serialize>(&mut self, message: T) {
        match self.writer.serialize(message) {
            Err(err) => log::error!("Serialize Error: {:?}", err),
            _ => self.writer.flush().unwrap(),
        }
    }
}

impl<W: std::io::Write> StreamProcessor<bitmex::Response> for CsvSink<W> {
    fn process(&mut self, msg: &bitmex::Response) {
        use bitmex::Response::*;
        match msg {
            Subscribe { subscribe, success } => {
                log::info!("Subscribed: {}: success: {}", subscribe, success)
            }
            i @ Info { .. } => log::info!("{:?}", i),
            e @ Error { .. } => log::error!("{:?}", e),
            TableData {
                table,
                action,
                data,
            } => {
                for row in data {
                    self.serialize(bitmex::TableRowAction { table, action, row })
                }
            }
        }
    }
}

impl<T> ws::Handler for Client<T>
where
    T: serde::de::DeserializeOwned,
{
    fn on_open(&mut self, _: ws::Handshake) -> ws::Result<()> {
        let subscribe = bitmex::Request::Subscribe {
            args: vec![
                bitmex::Table::Trade,
                //bitmex::Table::OrderBookL2,
                //bitmex::Table::OrderBookL2_25,
            ],
        };

        let ser = serde_json::to_string(&subscribe).unwrap();
        log::info!("Sending subscribe command: {:?}", ser);
        self.out.send(ser)
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        let payload: String = msg.into_text().unwrap();
        //println!("{}", payload);
        let resp: Result<T, serde_json::error::Error> = serde_json::from_str(&payload);
        match resp {
            Ok(resp) => self.tx.send(resp).unwrap(),
            Err(err) => log::error!("{}", err),
        }

        Ok(()) // never fail
    }

    fn on_error(&mut self, err: ws::Error) {
        log::error!("On Error, {}", err)
    }
}

fn main() {
    env_logger::init();
    let (tx, rx) = mpsc::channel::<bitmex::Response>();
    let mut csv_sink = CsvSink::new(std::io::stdout());

    thread::spawn(move || loop {
        match rx.recv() {
            Ok(resp) => csv_sink.process(&resp),
            Err(err) => log::error!("Channel receive error: {:?}", err),
        }
    });

    let bitmex_addr = "wss://www.bitmex.com/realtime";
    log::info!("Connecting to {}", bitmex_addr);
    if let Err(error) = ws::connect(bitmex_addr, |out| Client {
        out: out,
        tx: tx.clone(),
    }) {
        log::error!("Failed to create WebSocket due to: {:?}", error)
    }
}
