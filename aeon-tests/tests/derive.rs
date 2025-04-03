extern crate aeon;
extern crate aeon_derive;

#[cfg(test)]
mod tests {
    use aeon::{AeonDeserialize, AeonSerialize};
    use aeon_derive::{Deserialize, Serialize};
    use std::collections::HashMap;

    // TODO: #[aeon] for custom deserializing/serializing function for specific property?
    // TODO: maybe #[aeon(default)] to use Default::default when nothing was found etc
    // TODO: #[aeon(field = "like_this")] to change the field name to "like_this" when serialized

    macro_rules! map(
    ( $( $k:expr => $v:expr ),+ $(,)? ) => ( // $(,)? is to always allow trailing commas
        {
            let mut map = std::collections::HashMap::new();
            $(
                map.insert($k, $v);
            )+
            map
        }
    );
);

    #[derive(Deserialize, Serialize)]
    /// it's fine to have a comment here
    /// multiple even
    pub struct TestIgnore {
        /// it's fine to have a comment here
        /// multiple even
        pub it: String,
        pub it2: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct FirstPhase {
        pub it: String,
        pub it2: ::std::string::String,
        pub not: std::collections::HashMap<String, std::option::Option<HashMap<String, i64>>>,
    }

    #[test]
    pub fn test_deserialize() {
        let aeon = r#"
        it: "hello"
        it2: "hi \"there\""
        not: { cool: { it: 1 }, so_cool: nil }
        "#
        .to_string();
        let test = FirstPhase::from_aeon(aeon).unwrap();
        assert_eq!("hello", test.it);
        assert_eq!("hi \"there\"", test.it2);
        assert!(test.not.get("so_cool").unwrap().is_none());
        assert_eq!(
            &1,
            test.not
                .get("cool")
                .unwrap()
                .as_ref()
                .unwrap()
                .get("it")
                .unwrap()
        );
    }

    #[test]
    pub fn test_serialize() {
        let first = FirstPhase {
            it: "hello".into(),
            it2: "hi \"there\"".into(),
            not: map!(
                "cool".into() => Some(map!("it".into() => 1)),
                "so_cool".into() => None,
            ),
        };
        let test: String = first.to_aeon().unwrap().replace("    ", "\t");
        assert!(test.contains(r#"it: "hello""#));
        assert!(test.contains(r#"it2: "hi \"there\"""#));
        assert!(test.contains("not: "));
        assert!(test.contains("cool: "));
        assert!(test.contains("it: 1"));
        assert!(test.contains("so_cool: nil"));
    }

    #[derive(Serialize, Deserialize)]
    struct OtherDerive {
        veco: Vec<String>,
        do_it: bool,
    }

    #[derive(Serialize, Deserialize)]
    struct TestDerive {
        bytes: Vec<u8>,
        some_ip: String,
        thingy: HashMap<String, HashMap<String, OtherDerive>>,
        maybe: Option<bool>,
    }
    #[test]
    pub fn test_serialize2() {
        let test = TestDerive {
            bytes: vec![9, 1],
            some_ip: String::from("1.2.3.4"),
            thingy: HashMap::new(),
            maybe: None,
        };
        let serialized: String = test.to_aeon().unwrap().replace("    ", "\t");
        assert!(serialized.contains("some_ip: \"1.2.3.4\""));
        assert!(serialized.contains("bytes: [\n\t9,\n\t1\n]"));
        assert!(serialized.contains("thingy: {}"));
        assert!(serialized.contains("maybe: nil"));
        assert!(
            serialized.contains("@OtherDerive(veco, do_it)"),
            "{}",
            serialized
        );
    }

    #[test]
    pub fn test_deserialize2() {
        let aeon = "@OtherDerive(veco, do_it) some_ip: \"5.6.7.8\"\nbytes: [5,4,6]\n#some_thing: nil\nsome_nested_thing: [\"o hi\", \"a thingy\"] thingy:{snappy:{scrappy:OtherDerive([\"hi world\"], false)}} maybe: true".to_string();
        let test = TestDerive::from_aeon(aeon).unwrap();
        assert_eq!("5.6.7.8", test.some_ip);
        assert_eq!(vec![5u8, 4u8, 6u8], test.bytes);
        assert!(test.maybe.unwrap());
        assert_eq!(
            "hi world",
            test.thingy
                .get("snappy")
                .unwrap()
                .get("scrappy")
                .unwrap()
                .veco
                .first()
                .unwrap()
        )
    }
}
