#[cfg(test)]
mod tests {
    use aeon::convert::{AeonObjectConvert, AeonConvert};

    #[test]
    pub fn deserialize_map_with_both_quoted_and_not_quoted_keys() {
        let aeon = r#"map: {test: 1, "two": 2}"#.into();
        let ser = aeon::deserialize(aeon).expect("failed to deserialize");

        assert_eq!(ser.get_path("map/test").int(), Some(1));
        assert_eq!(ser.get_path("map/two").int(), Some(2));
    }

    #[test]
    pub fn deserialize_double() {
        let aeon = r#"doub: 2.10"#.into();
        let ser = aeon::deserialize(aeon).expect("failed to deserialize");

        assert_eq!(ser.get("doub").double(), Some(2.10));
    }

    #[test]
    pub fn deserialize_ip() {
        let aeon = r#"ip: 127.2.3.4"#.into();
        let ser = aeon::deserialize(aeon).expect("failed to deserialize");

        assert_eq!(ser.get("ip").ip(),
                   Some(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127,2,3,4))));
    }

    #[test]
    pub fn deserialize_ip_and_get_as_string() {
        let aeon = r#"ip: 127.2.3.4"#.into();
        let ser = aeon::deserialize(aeon).expect("failed to deserialize");

        assert_eq!(ser.get("ip").ip_str(),
                   Some(String::from("127.2.3.4")));
    }
}
