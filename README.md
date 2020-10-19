# Aeon

⚠ **Works, but isn't production ready**

Awfully exciting object notation

### Example file
```
@server(id, name, ip, port)
servers: [
	server(1, "test", [127,0,0,1], 7171),
	server(2, "production", [0,0,0,0], 8080),
]
```

### Usage
```rust
/* using derive macro */
use aeon::convert_panic::*;
use aeon::*;
use aeon_derive::{Deserialize,Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Heuristic{
    pub value: String,
    pub weight: i32,
} 

#[derive(Serialize, Deserialize, Debug)]
pub struct WithHeuristics {
	pub something: Vec<Heuristic>,
	pub else: bool,
}

// deserialize:
let heuristics = WithHeuristics::from_aeon(some aeon string here);
// serialize:
println!("{}", WithHeuristics::to_aeon(&heuristics));
// would print something similar to:
/*
@heuristic(value, weight)

something: [
	heuristic("some_name", 10),
	heuristic("some_other_name", 19),
]

else: false
*/


/* typing it out manually */
use aeon::*;
use aeon::convert_panic::*; // there's also aeon::convert::* if you prefer Option<T> over panics

let servers = aeon::deserialize(data).get("servers").list();
println!("{:?}", servers);
// there's also get_path("path/to/value") functions
```

### Comments
Comments start with a '#' symbol.
```
# comments are allowed on their own lines
thing: "text" # and at the end of lines
```

Comments are _not_ serialized when performing deserializing -> serializing, this may change in the future

### Supported types
- Lists - ["One", 2, 3]
- Maps - {"one": 1, "a": "b"}
- Bools (true/false are the only valid identifiers for bools)
- Integers
- Decimal numbers - Use a dot '.' as the decimal separator
- Strings - Use double quotes, e.g. "this is a string"

### Macros
Macros are parsed as HashMap<String, AeonValue>.

Macros start with an '@' symbol, followed by an identifier and a list of arguments.

E.g. @identifier(argument1, argument2, argument3)

Macros need to be defined before they are used, preferably at the start of the file, before any variables.

Macro identifiers can also be used as variable identifiers.
