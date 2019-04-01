#[macro_use]
extern crate enum_display_derive;
extern crate chrono;
extern crate csv;
extern crate env_logger;
extern crate ws;

mod wire;
use wire::bitmex;

struct Client {
    out: ws::Sender,
}

fn main() {
    env_logger::init();
    let bitmex_addr = "wss://www.bitmex.com/realtime";
    log::info!("Connecting to {}", bitmex_addr);
    if let Err(error) = ws::connect(bitmex_addr, |out| Client { out: out }) {
        log::error!("Failed to create WebSocket due to: {:?}", error)
    }
}

impl ws::Handler for Client {
    fn on_open(&mut self, _: ws::Handshake) -> ws::Result<()> {
        use bitmex::*;
        let subscribe = Request::Subscribe {
            args: vec![Table::Trade, Table::OrderBookL2, Table::OrderBookL2_25],
        };

        let ser = serde_json::to_string(&subscribe).unwrap();
        log::info!("Subscribing: {:?}", ser);
        self.out.send(ser)
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        let payload: String = msg.into_text().unwrap();
        let resp: serde_json::Result<bitmex::Response> = serde_json::from_str(&payload);
        match resp {
            Ok(resp) => process_response(&resp),
            Err(err) => log::error!("{}", err),
        }

        Ok(()) // never fail
    }

    fn on_error(&mut self, err: ws::Error) {
        log::error!("On Error, {}", err)
    }
}

fn process_response(resp: &bitmex::Response) {
    use bitmex::Response::*;
    match resp {
        TableData {
            table,
            action,
            data,
        } => {
            for row in data {
                process_action(bitmex::TableRowAction { table, action, row })
            }
        }
        Info {
            info,
            version,
            timestamp,
        } => log::info!(
            "Info: {}, version: {}, timestmap: {}",
            info,
            version,
            timestamp
        ),
        Subscribe { subscribe, success } => log::info!("Subscribed: {}: {}", subscribe, success),
        Error { status, error, .. } => log::error!("error: {}: {}", status, error),
    }
}

fn process_action(row: bitmex::TableRowAction) {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'|')
        .has_headers(false)
        .from_writer(std::io::stdout());

    match writer.serialize(row) {
        Err(err) => log::error!("{}", err),
        _ => (),
    }
    ()
}
