extern crate aeon;
extern crate aeon_derive;


#[cfg(test)]
mod tests {
    use aeon_derive::{Serialize,Deserialize};
	use aeon::{AeonSerialize,AeonDeserialize};
    use aeon::convert_panic::{AeonConvert, AeonObjectConvert};

    #[derive(Serialize,Deserialize)]
    struct TestDerive {
        bytes: Vec<u8>,
        // TODO: #[custom] for custom deserializing/serializing function for specific property?
        some_ip: String,
    }

    #[test]
    pub fn test_serialize()
    {
        let test = TestDerive {
            bytes: vec![9,1],
            some_ip: String::from("1.2.3.4"),
        };
        let serialized : String = test.to_aeon().replace("    ", "\t");
        assert!(serialized.contains("some_ip: \"1.2.3.4\""));
        assert!(serialized.contains("bytes: [\n\t9,\n\t1\n]"));
    }

    #[test]
    pub fn test_deserialize()
    {
        let aeon = "some_ip: \"5.6.7.8\"\nbytes: [5,4,6]".to_string();
        let test = TestDerive::from_aeon(aeon);
        assert_eq!("5.6.7.8", test.some_ip);
        assert_eq!(vec![5u8,4u8,6u8], test.bytes);
    }
}
