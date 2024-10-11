use rusqlite::{Connection, Error};

use crate::xml_parser::Price;

pub struct Database {
    pub connection: Connection
}

impl Database {
    pub fn get_connection() -> Self {
        let conn = Connection::open("elcron.db").unwrap();
        Self {
            connection: conn
        }
    }
    pub fn create_schema(&self) -> Result<usize, Error> {
        self.connection.execute("
            CREATE TABLE IF NOT EXISTS spot (
                id      INTEGER PRIMARY KEY,
                date    TEXT,
                hour    INTEGER,
                price   REAL,
                UNIQUE(date, hour)
            )
        ", ())
    }
    pub fn insert_price(&self, price: &Price) -> Result<usize, Error> {
        self.connection.execute("INSERT INTO spot (date, hour, price) VALUES (?1, ?2, ?3)", (&price.date, &price.hour, &price.price))
    }
    pub fn _get_spot_prices(&self) -> Vec<Price> {
        let mut prices = vec![];
        let mut statement = self.connection.prepare("SELECT date, hour, price FROM spot").unwrap();
        let price_iter = statement.query_map([], |row| {
            Ok(Price {
                date: row.get(0)?,
                hour: row.get(1)?,
                price: row.get(2)?
            })
        });
        for p in price_iter.unwrap() {
            prices.push(p.unwrap());
        }
        return prices;
    }
}
