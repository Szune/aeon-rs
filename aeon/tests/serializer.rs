#[cfg(test)]
mod tests {
	fn like(expected: &str, actual: &str) -> bool {
		let (mut expected_iter, mut actual_iter) = 
				(expected.chars(), actual.chars());

		loop {
			let (mut exp, mut act) = 
				(expected_iter.next(), actual_iter.next());
			while let Some(v) = exp {
				if !v.is_whitespace() {
					break;
				}
				exp = expected_iter.next();
			}
			while let Some(v) = act {
				if !v.is_whitespace() {
					break;
				}
				act = actual_iter.next();
			}

			if exp != act {
				return false;
			}

			if exp.is_none() || act.is_none() {
				return true;
			}
		};
	}

    use aeon::object::{AeonObject, AeonProperty, Macro};
    use aeon::value::{AeonValue};
    use aeon::map;

    #[test]
    pub fn serialize_using_macros() {
        let mut aeon = AeonObject::new();
        let aeon_value = AeonProperty::new("char".into(), AeonValue::Map(map![
           "name".into() => AeonValue::String("erki".into()),
           "world".into() => AeonValue::Integer(1),
           "double".into() => AeonValue::Double(139.3567),
           "or_nothing".into() => AeonValue::Nil,
        ]));
        aeon.add_macro(Macro::new("character".into(), vec![
            "name".into(),
            "world".into(),
            "double".into(),
            "or_nothing".into(),
        ]));
        aeon.add_property(aeon_value);
        let serialized = aeon::serialize(aeon);
		println!("{}", serialized);
        assert!(like("@character(name, world, double, or_nothing) char: character(\"erki\", 1, 139.3567, nil)", serialized.as_str()));
    }

    #[test]
    pub fn serialize_using_nested_macros() {
        let mut aeon = AeonObject::new();
        let aeon_value = AeonProperty::new("char".into(), AeonValue::Map(map![
           "name".into() => AeonValue::String("erki".into()),
           "world".into() => AeonValue::Integer(1),
           "double".into() => AeonValue::Double(139.3567),
           "or_nothing".into() => AeonValue::Map(map![
               "name".into() => AeonValue::String("unused".into()),
               "world".into() => AeonValue::Integer(-53),
               "double".into() => AeonValue::Double(-11.38),
               "or_nothing".into() => AeonValue::Nil,
           ]),
        ]));
        aeon.add_macro(Macro::new("character".into(), vec![
            "name".into(),
            "world".into(),
            "double".into(),
            "or_nothing".into(),
        ]));
        aeon.add_property(aeon_value);
        let serialized = aeon::serialize(aeon);
        assert!(like("@character(name, world, double, or_nothing)\nchar: character(\"erki\", 1, 139.3567, character(\"unused\", -53, -11.38, nil))\n", serialized.as_str()));
    }

    #[test]
    pub fn serialize_map_property() {
        let mut aeon = AeonObject::new();
        let aeon_value = AeonProperty::new("character".into(), AeonValue::Map(map![
           "name".into() => AeonValue::String("erki".into()),
           "world".into() => AeonValue::Integer(1),
           "double".into() => AeonValue::Double(139.3567),
           "or_nothing".into() => AeonValue::Nil,
        ]));
        aeon.add_property(aeon_value);
        let serialized = aeon::serialize(aeon);
        // TODO: regex or rewrite serialize implementation to be more testable
        // or just don't test the entire serialization and instead its parts
        assert!(serialized.starts_with("character: {"));
        assert!(serialized.ends_with("}\n\n"));
        assert!(serialized.contains(r#"name: "erki""#));
        assert!(serialized.contains("world: 1"));
        assert!(serialized.contains("double: 139.3567"));
        assert!(serialized.contains("or_nothing: nil"));
        assert!(serialized.contains(','));
    }

    #[test]
    pub fn serialize_map_property_key_that_is_not_a_valid_identifier() {
        let mut aeon = AeonObject::new();
        let aeon_value = AeonProperty::new("job".into(), AeonValue::Map(map![
           "9to5".into() => AeonValue::Bool(true),
           "NineToFive".into() => AeonValue::Bool(true),
        ]));
        aeon.add_property(aeon_value);
        let serialized = aeon::serialize(aeon);
        assert!(serialized.starts_with("job: {"));
        assert!(serialized.ends_with("}\n\n"));
        assert!(serialized.contains(r#""9to5": true"#));
        assert!(serialized.contains(r#"NineToFive: true"#));
        assert!(serialized.contains(','));
    }

    #[test]
    pub fn serialize_list_of_strings_property() {
        let mut aeon = AeonObject::new();
        let aeon_value = AeonProperty::new("characters".into(), AeonValue::List(vec![
           AeonValue::String("erki".into()),
           AeonValue::String("persiko".into()),
           AeonValue::String("frukt".into()),
           AeonValue::String("152436.13999".into()),
        ]));
        aeon.add_property(aeon_value);
        let ser = aeon::serialize(aeon);
        assert!(like("characters: [\"erki\", \"persiko\", \"frukt\", \"152436.13999\"]\n", ser.as_str()));
    }

    #[test]
    pub fn serialize_string_property() {
        let mut aeon = AeonObject::new();
        let aeon_value = AeonProperty::new("character".into(), AeonValue::String("erki isthename".into()));
        aeon.add_property(aeon_value);
        let ser = aeon::serialize(aeon);
        assert_eq!("character: \"erki isthename\"\n\n", ser);
    }

    #[test]
    pub fn serialize_list_of_ints() {
        let mut aeon = AeonObject::new();
        let aeon_value = AeonProperty::new("values".into(), AeonValue::List(vec![
           AeonValue::Integer(5),
           AeonValue::Integer(3),
           AeonValue::Integer(2),
           AeonValue::Integer(1),
        ]));
        aeon.add_property(aeon_value);
        let ser = aeon::serialize(aeon);
        assert_eq!("values: [\n    5,\n    3,\n    2,\n    1\n]\n\n", ser);
    }


    #[test]
    pub fn serialize_macros() {
        let mut aeon = AeonObject::new();
        aeon.add_macro(Macro::new("character".into(), vec!["name".into(), "world".into()]));
        let ser = aeon::serialize(aeon);
        assert!(like("@character(name, world)\n", ser.as_str()));
    }
}
