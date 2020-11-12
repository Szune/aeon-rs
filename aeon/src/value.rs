use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum AeonValue {
    Nil,
    Bool(bool),
    String(String),
    Integer(i64),
    Double(f64),
    Map(HashMap<String,AeonValue>),
    List(Vec<AeonValue>),
    Ip(std::net::IpAddr),
}
