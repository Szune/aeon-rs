# Aeon

⚠ **Works, but isn't production ready**

⚠ **aeon-derive is barely started**

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
let servers = aeon::deserialize(data).unwrap().get("servers").unwrap().list().unwrap();
println!("{:?}", servers);
// there's also get("path/to/value") functions, currently they do not support indexing though
```

### Comments
Comments start with a '#' symbol.
```
# comments are allowed on their own lines
thing: "text" # and at the end of lines
```

### Supported types
- Lists - ["One", 2, 3]
- Maps - {"one": 1, "a": "b"}
- Integers
- Decimal numbers - Use a dot '.' as the decimal separator
- Strings - Use double quotes, e.g. "this is a string"

### Macros
Macros are parsed as HashMap<String, AeonValue>.

Macros start with an '@' symbol, followed by an identifier and a list of arguments.

E.g. @identifier(argument1, argument2, argument3)

Macros need to be defined before they are used, preferably at the start of the file, before any variables.

Macro identifiers can also be used as variable identifiers.
