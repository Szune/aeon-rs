#[cfg(test)]
mod tests {
    use aeon::convert::{AeonObjectConvert, AeonConvert};
    use aeon::value::AeonValue;

    #[test]
    pub fn deserialize_map_with_both_quoted_and_not_quoted_keys() {
        let mut aeon = r#"map: {test: 1, "two": 2}"#.into();
        let ser = aeon::deserialize(aeon).expect("failed to deserialize");

        assert_eq!(ser.get_path("map/test").int(), Some(1));
        assert_eq!(ser.get_path("map/two").int(), Some(2));
    }
}
