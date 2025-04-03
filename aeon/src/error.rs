use std::fmt::{Display, Formatter};

macro_rules! str_variants(
    (pub enum $name:ident {
        $($variant:ident),*
        $(,)?
    }) => {
        #[derive(Debug)]
        pub enum $name {
            $($variant,)*
        }

        impl $name {
            pub fn variant_name(&self) -> &'static str {
                match self {
                    $($name::$variant => stringify!($variant),)*
                }
            }
        }
    }
);

str_variants! {
    pub enum AeonSerializeErrorCode {
        ConversionFailed,
        SerializationFailed,
    }
}

str_variants! {
    pub enum AeonDeserializeErrorCode {
        ConversionFailed,
        DeserializationFailed,
        LexingFailed,
    }
}

#[derive(Debug)]
pub struct AeonSerializeError {
    pub code: AeonSerializeErrorCode,
    pub message: String,
}

#[derive(Debug)]
pub struct AeonDeserializeError {
    pub code: AeonDeserializeErrorCode,
    pub message: String,
}

impl Display for AeonSerializeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code.variant_name(), self.message)
    }
}

impl Display for AeonDeserializeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code.variant_name(), self.message)
    }
}

impl AeonDeserializeError {
    pub fn lexing(message: String) -> Self {
        Self {
            code: AeonDeserializeErrorCode::LexingFailed,
            message,
        }
    }

    pub fn deserialization(message: String) -> Self {
        Self {
            code: AeonDeserializeErrorCode::DeserializationFailed,
            message,
        }
    }

    pub fn conversion(message: String) -> Self {
        Self {
            code: AeonDeserializeErrorCode::ConversionFailed,
            message,
        }
    }
}
