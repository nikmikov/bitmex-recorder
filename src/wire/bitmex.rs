use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug, Display)]
#[serde(rename_all = "camelCase")]
pub enum Table {
    Trade,
    OrderBookL2,
    OrderBookL2_25,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "op")]
pub enum Request {
    Subscribe { args: Vec<Table> },
}

#[derive(Serialize, Deserialize, Debug, Display)]
#[serde(rename_all = "lowercase")]
#[serde(deny_unknown_fields)]
pub enum TableAction {
    Partial,
    Update,
    Insert,
    Delete,
}

#[derive(Serialize, Deserialize, Debug, Display)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Serialize, Deserialize, Debug, Display)]
pub enum TickDirection {
    ZeroPlusTick,
    PlusTick,
    ZeroMinusTick,
    MinusTick,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Response {
    Info {
        info: String,
        version: String,
        timestamp: DateTime<Utc>,
    },
    Subscribe {
        subscribe: Table,
        success: bool,
    },
    Error {
        status: u16,
        error: String,
        request: Request,
    },
    TableData {
        table: Table,
        action: TableAction,
        data: Vec<TableRow>,
    },
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(untagged)]
#[serde(deny_unknown_fields)]
pub enum TableRow {
    #[serde(rename_all = "camelCase")]
    Trade {
        #[serde(default = "current_timestamp")]
        processed: DateTime<Utc>,
        timestamp: DateTime<Utc>,
        symbol: String,
        side: Side,
        size: u64,
        price: f64,
        tick_direction: TickDirection,
        #[serde(rename = "trdMatchID")]
        trade_match_id: String,
        gross_value: Option<u64>,
        home_notional: Option<f64>,
        foreign_notional: Option<f64>,
    },
    #[serde(rename_all = "camelCase")]
    Order {
        #[serde(default = "current_timestamp")]
        processed: DateTime<Utc>,
        symbol: String,
        id: u64,
        side: Side,
        size: Option<u64>,
        price: Option<f64>,
    },
}

#[derive(Serialize, Debug)]
pub struct TableRowAction<'a> {
    pub table: &'a Table,
    pub action: &'a TableAction,
    pub row: &'a TableRow,
}

fn current_timestamp() -> DateTime<Utc> {
    Utc::now()
}
