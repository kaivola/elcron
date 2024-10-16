use core::fmt;

use log::info;
use xml::reader::{EventReader, XmlEvent};

#[derive(Debug)]
pub struct Price {
    pub date: String,
    pub hour: u8,
    pub price: f64,
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, "(date={}, hour={}, price={:.2})",
            self.date, self.hour, self.price
        )
    }
}

pub fn parse_price_xml(xml: &str) -> Vec<Price> {
    info!("Starting to parse XML");
    let mut parser = EventReader::from_str(xml);
    let mut prices: Vec<Price> = Vec::new();
    let mut date = String::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => match name.local_name.as_str() {
                "end" => match parser.next() {
                    Ok(XmlEvent::Characters(date_str)) => {
                        date = date_str.split("T").take(1).collect();
                    }
                    Err(e) => panic!("Error parsing date from the xml: {e}"),
                    _ => {}
                },
                "Point" => {
                    let mut price_struct = Price {
                        date: date.clone(),
                        hour: 0,
                        price: 0.0,
                    };
                    loop {
                        match parser.next() {
                            Ok(XmlEvent::StartElement { name, .. }) => {
                                match name.local_name.as_str() {
                                    "position" => if let Ok(XmlEvent::Characters(position)) = parser.next() {
                                        let mut pos = position.parse().unwrap();
                                        if pos == 24 {
                                            pos = 0;
                                            price_struct.date =
                                                chrono::NaiveDate::parse_from_str(
                                                    &price_struct.date,
                                                    "%Y-%m-%d",
                                                )
                                                .unwrap()
                                                .succ_opt()
                                                .unwrap()
                                                .to_string();
                                        }
                                        price_struct.hour = pos;
                                    },
                                    "price.amount" => if let Ok(XmlEvent::Characters(price)) = parser.next() {
                                        price_struct.price = price.parse::<f64>().unwrap() / 10.0;
                                    },
                                    _ => {}
                                }
                            }
                            Ok(XmlEvent::EndElement { name }) => {
                                if name.local_name == "Point" {
                                    prices.push(price_struct);
                                    break;
                                }
                            }
                            Ok(XmlEvent::EndDocument) => {
                                break;
                            }
                            Err(e) => panic!("Error: {e}"),
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            Ok(XmlEvent::EndDocument) => {
                break;
            }
            Err(e) => panic!("Error: {e}"),
            _ => {}
        }
    }
    info!("Finished parsing XML - found {} prices", prices.len());
    prices
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_xml_parsing() {
        let xml = r#"
            <root>
                <end>2021-01-01T00:00:00Z</end>
                <Point>
                    <position>1</position>
                    <price.amount>10.0</price.amount>
                </Point>
                <Point>
                    <position>2</position>
                    <price.amount>20.0</price.amount>
                </Point>
                <Point>
                    <position>3</position>
                    <price.amount>30.0</price.amount>
                </Point>
                <end>2021-01-02T00:00:00Z</end>
                <Point>
                    <position>4</position>
                    <price.amount>40.0</price.amount>
                </Point>
                <Point>
                    <position>5</position>
                    <price.amount>50.0</price.amount>
                </Point>
                <DoNotInclude>
                    <position>6</position>
                    <price.amount>60.0</price.amount>
                </DoNotInclude>
            </root>
        "#;
        let prices = parse_price_xml(xml);
        assert_eq!(prices.len(), 5);
        assert_eq!(prices[0].date, "2021-01-01");
        assert_eq!(prices[0].hour, 1);
        assert_eq!(prices[0].price, 1.0);
        assert_eq!(prices[4].date, "2021-01-02");
        assert_eq!(prices[4].hour, 5);
        assert_eq!(prices[4].price, 5.0);
    }
}
