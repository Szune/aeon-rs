#[macro_export]
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
