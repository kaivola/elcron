use core::fmt;

use xml::reader::{EventReader, XmlEvent};

#[derive(Debug)]
pub struct Price {
    pub date: String,
    pub hour: u8,
    pub price: f64
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(date={}, hour={}, price={})", self.date, self.hour, self.price)
    }
}

pub fn parse_price_xml(xml: &str) -> Vec<Price> {
    println!("Starting to parse XML");
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
                    Err(e) => {
                        eprintln!("Error parsing date: {e}");
                        break;
                    }
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
                                    "position" => match parser.next() {
                                        Ok(XmlEvent::Characters(position)) => {
                                            price_struct.hour = position.parse().unwrap();
                                        }
                                        _ => {}
                                    },
                                    "price.amount" => match parser.next() {
                                        Ok(XmlEvent::Characters(price)) => {
                                            price_struct.price = price.parse().unwrap();
                                        }
                                        _ => {}
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
                            Err(e) => {
                                eprintln!("Error: {e}");
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            Ok(XmlEvent::EndDocument) => {
                break;
            }
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
            _ => {}
        }
    }
    println!("Finished parsing XML - found {} prices", prices.len());
    return prices;
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
                <asd>
                    <position>6</position>
                    <price.amount>60.0</price.amount>
                </asd>
            </root>
        "#;
        let prices = parse_price_xml(xml);
        assert_eq!(prices.len(), 5);
        assert_eq!(prices[0].date, "2021-01-01");
        assert_eq!(prices[0].hour, 1);
        assert_eq!(prices[0].price, 10.0);
        assert_eq!(prices[4].date, "2021-01-02");
        assert_eq!(prices[4].hour, 5);
        assert_eq!(prices[4].price, 50.0);
    }
}