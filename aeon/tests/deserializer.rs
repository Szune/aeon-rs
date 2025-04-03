#[cfg(test)]
mod tests {
    use aeon::map;

    /// Implements [`aeon::AeonDeserialize`] for the specified type by delegating to its [`aeon::AeonDeserializeProperty`] implementation
    ///
    macro_rules! impl_deserialize {
        ($name:path) => {
            impl aeon::AeonDeserialize for $name {
                fn from_aeon(s: String) -> aeon::DeserializeResult<Self> {
                    use aeon::AeonDeserializeProperty;
                    aeon::deserialize(s).and_then(|doc| Self::from_property(doc.into()))
                }
            }
        };
    }

    macro_rules! impl_serialize {
        ($name:path) => {
            impl aeon::AeonSerialize for $name {
                fn to_aeon(&self) -> aeon::SerializeResult<String> {
                    use aeon::document::AeonDocument;
                    let mut doc = AeonDocument::try_from_object(self.to_aeon_value()?).unwrap();
                    doc.set_macros(Self::create_macros(false));
                    aeon::serialize(&doc)
                }

                fn to_aeon_value(&self) -> aeon::SerializeResult<aeon::value::AeonValue> {
                    use aeon::AeonSerializeProperty;
                    self.serialize_property()
                }

                fn create_macros(
                    insert_self: bool,
                ) -> std::collections::HashMap<String, aeon::document::AeonMacro> {
                    use aeon::AeonSerializeProperty;
                    Self::create_property_macros(insert_self)
                }
            }
        };
    }

    /// # Examples
    ///
    /// ```
    /// struct Thing {
    ///     cool: u32,
    ///     maybe_cool: Option<i32>,
    /// }
    ///
    /// fn ser_props() -> AeonValue {
    ///     let mut obj = std::collections::HashMap::<String, AeonValue>::new();
    ///     aeon_prop_ser!(obj => self.cool);
    ///     aeon_prop_ser!(maybe obj => self.maybe_cool);
    ///     AeonValue::Object(obj)
    /// }
    /// ```
    macro_rules! aeon_prop_ser {
        (maybe $obj:expr => $prop:expr) => {
            $obj.insert(
                stringify!($prop).replace("self.", "").into(),
                $prop.serialize_property_or_nil(),
            );
        };
        ($obj:expr => $prop:expr) => {
            $obj.insert(
                stringify!($prop).replace("self.", "").into(),
                $prop.serialize_property().unwrap(),
            );
        };
    }

    #[test]
    pub fn deserialize_map_with_both_quoted_and_not_quoted_keys() {
        use aeon::convert::{AeonConvert, AeonObjectConvert};
        let aeon = r#"map: {test: 1, "two": 2}"#.into();
        let ser = aeon::deserialize(aeon).ok().expect("failed to deserialize");

        assert_eq!(ser.get_path("map/test").int(), Some(1));
        assert_eq!(ser.get_path("map/two").int(), Some(2));
    }

    #[test]
    pub fn deserialize_double() {
        use aeon::convert::{AeonConvert, AeonObjectConvert};
        let aeon = r#"doub: 2.10"#.into();
        let ser = aeon::deserialize(aeon).ok().expect("failed to deserialize");

        assert_eq!(ser.get("doub").double(), Some(2.10));
    }

    // TODO: make it easier to implement these functions with macros?
    // TODO: i.e. in addition to the proc macros

    struct NestedDerive {
        cool: u32,
        maybe_cool: Option<i32>,
    }

    impl aeon::AeonDeserializeProperty for NestedDerive {
        fn from_property(field: aeon::value::AeonValue) -> aeon::DeserializeResult<Self> {
            use aeon::convert::AeonConvert;
            Ok(Self {
                cool: field.get("cool").of().unwrap(),
                maybe_cool: field.get("maybe_cool").of(),
            })
        }
    }

    impl aeon::AeonSerializeProperty for NestedDerive {
        fn serialize_property(&self) -> aeon::SerializeResult<aeon::value::AeonValue> {
            use aeon::value::AeonValue;
            let mut obj = std::collections::HashMap::<String, AeonValue>::new();
            obj.insert("cool".into(), self.cool.serialize_property()?);
            obj.insert("maybe_cool".into(), self.maybe_cool.serialize_property()?);
            Ok(AeonValue::Object(obj))
        }

        fn create_property_macros(
            insert_self: bool,
        ) -> std::collections::HashMap<String, aeon::document::AeonMacro> {
            use aeon::document::AeonMacro;
            let mut macros = std::collections::HashMap::<String, AeonMacro>::new();
            if insert_self {
                macros.insert(
                    "nestedstruct".into(),
                    AeonMacro::new_cloned("nestedstruct", vec!["cool", "maybe_cool"]),
                );
            }
            macros.extend(u32::create_property_macros(true));
            macros
        }
    }

    struct TestDerive {
        bytes: Vec<u8>,
        // TODO: #[custom] for custom deserializing/serializing function for specific property?
        some_ip: String,
        some_thing: Option<String>,
        some_nested_thing: Option<Vec<String>>,
        some_hash_map: std::collections::HashMap<String, aeon::value::AeonValue>,
        opt_hash_map: Option<std::collections::HashMap<String, aeon::value::AeonValue>>,
        some_hash_map_with_other_values: std::collections::HashMap<String, NestedDerive>,
        nested_struct: NestedDerive,
    }

    impl_deserialize!(TestDerive);

    impl_serialize!(TestDerive);

    impl aeon::AeonDeserializeProperty for TestDerive {
        fn from_property(field: aeon::value::AeonValue) -> aeon::DeserializeResult<Self> {
            use aeon::convert::AeonConvert;
            Ok(Self {
                bytes: field
                    .get("bytes")
                    .map(aeon::AeonDeserializeProperty::from_property)
                    .transpose()?
                    .unwrap(),
                some_ip: field
                    .get("some_ip")
                    .map(aeon::AeonDeserializeProperty::from_property)
                    .transpose()?
                    .unwrap(),
                some_thing: aeon::convert::maybe(field.get("some_thing"))?,
                some_nested_thing: field
                    .get("some_nested_thing")
                    .map(aeon::AeonDeserializeProperty::from_property)
                    .transpose()?,
                some_hash_map: field
                    .get("some_hash_map")
                    .map(aeon::AeonDeserializeProperty::from_property)
                    .transpose()?
                    .unwrap(), //.object().unwrap(),
                opt_hash_map: field.get("opt_hash_map").object(),
                some_hash_map_with_other_values: field
                    .get("some_hash_map_with_other_values")
                    .map(aeon::AeonDeserializeProperty::from_property)
                    .transpose()?
                    .unwrap(),
                nested_struct: field
                    .get("nested_struct")
                    .map(aeon::AeonDeserializeProperty::from_property)
                    .transpose()?
                    .unwrap(),
            })
        }
    }

    impl aeon::AeonSerializeProperty for TestDerive {
        fn serialize_property(&self) -> aeon::SerializeResult<aeon::value::AeonValue> {
            use aeon::value::AeonValue;
            let mut obj = std::collections::HashMap::<String, AeonValue>::new();
            obj.insert("bytes".into(), self.bytes.serialize_property()?);
            obj.insert("some_ip".into(), self.some_ip.serialize_property()?);
            obj.insert("some_thing".into(), self.some_thing.serialize_property()?);
            obj.insert(
                "some_nested_thing".into(),
                self.some_nested_thing.serialize_property()?,
            );
            obj.insert(
                "some_hash_map".into(),
                self.some_hash_map.serialize_property()?,
            );
            obj.insert(
                "opt_hash_map".into(),
                self.opt_hash_map.serialize_property()?,
            );
            obj.insert(
                "some_hash_map_with_other_values".into(),
                self.some_hash_map_with_other_values.serialize_property()?,
            );
            obj.insert(
                "nested_struct".into(),
                self.nested_struct.serialize_property()?,
            );
            Ok(AeonValue::Object(obj))
        }

        fn create_property_macros(
            insert_self: bool,
        ) -> std::collections::HashMap<String, aeon::document::AeonMacro> {
            use aeon::document::AeonMacro;
            let mut macros = std::collections::HashMap::<String, AeonMacro>::new();
            if insert_self {
                macros.insert(
                    "testderive".into(),
                    AeonMacro::new_cloned(
                        "testderive",
                        vec![
                            "bytes",
                            "some_ip",
                            "some_thing",
                            "some_nested_thing",
                            "some_hash_map",
                            "opt_hash_map",
                            "some_hash_map_with_other_values",
                            "nested_struct",
                        ],
                    ),
                );
            }

            macros.extend(NestedDerive::create_property_macros(true));
            macros.extend(u8::create_property_macros(true));
            macros.extend(String::create_property_macros(true));
            macros
        }
    }

    macro_rules! assert_contains (
            ($s:expr, $e:literal) => {
                assert!($s.contains($e), "Did not find {} in:\n{}", stringify!($e), $s)
            }
        );

    #[test]
    pub fn test_serialize() {
        use aeon::{value::AeonValue, AeonSerialize};
        let test = TestDerive {
            bytes: vec![9, 1],
            some_ip: String::from("1.2.3.4"),
            some_thing: None,
            some_nested_thing: Some(vec!["elo".to_string(), "o no".to_string()]),
            some_hash_map: map!("summits".into() => AeonValue::Integer(987)),
            opt_hash_map: None,
            some_hash_map_with_other_values: map!("thingy".into() => NestedDerive {
                cool: 001,
                maybe_cool: None
            }),
            nested_struct: NestedDerive {
                cool: 777,
                maybe_cool: Some(5),
            },
        };
        let serialized: String = TestDerive::to_aeon(&test)
            .unwrap()
            .replace(['\t', ' ', '\n', '\r'], "");
        assert_contains!(serialized, "some_ip:\"1.2.3.4\"");
        assert_contains!(serialized, "some_ip:\"1.2.3.4\"");
        assert_contains!(serialized, "bytes:[9,1]");
        assert_contains!(serialized, "some_thing:nil");
        assert_contains!(serialized, "some_nested_thing:[\"elo\",\"ono\"]");
        assert_contains!(serialized, "some_hash_map:{summits:987}");
        assert_contains!(serialized, "opt_hash_map:nil");
        assert_contains!(
            serialized,
            "some_hash_map_with_other_values:{thingy:nestedstruct(1,nil)}"
        );
        assert_contains!(serialized, "nested_struct:nestedstruct(777,5)");
    }

    #[test]
    pub fn test_deserialize() {
        use aeon::convert::AeonConvert;
        use aeon::AeonDeserialize;
        let aeon = "@nestedstruct(cool, maybe_cool)some_ip: \"5.6.7.8\"\nbytes: [5,4,6,]\nsome_thing: nil\nsome_nested_thing: [\"o hi \u{2714}\", \"a thingy\"]\nsome_hash_map: { summits: 987 }\nopt_hash_map: nil\nsome_hash_map_with_other_values:{thingy:nestedstruct(321,3310)}\nnested_struct: { \"cool\": 7171}".to_string();
        let test = TestDerive::from_aeon(aeon).unwrap();
        assert_eq!("5.6.7.8", test.some_ip);
        assert_eq!(vec![5u8, 4u8, 6u8], test.bytes);
        assert_eq!(None, test.some_thing);
        assert_eq!(
            Some(vec!["o hi ✔".to_string(), "a thingy".to_string()]),
            test.some_nested_thing
        );
        assert_eq!(
            3310,
            test.some_hash_map_with_other_values
                .get("thingy")
                .unwrap()
                .maybe_cool
                .unwrap()
        );
        assert_eq!(
            321,
            test.some_hash_map_with_other_values
                .get("thingy")
                .unwrap()
                .cool
        );
        assert_eq!(
            987,
            test.some_hash_map
                .get("summits")
                .unwrap()
                .clone()
                .int()
                .unwrap()
        );
        assert!(matches!(test.opt_hash_map, None));
    }
}
