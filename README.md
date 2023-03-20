# vmf_parser_nom

[![Crates.io](https://img.shields.io/crates/v/vmf_parser_nom)](https://crates.io/crates/vmf_parser_nom)
[![docs.rs](https://img.shields.io/docsrs/vmf_parser_nom)](https://docs.rs/vmf_parser_nom/latest/vmf_parser_nom)

A parser for the Valve map format written in Rust.
Also a provided convience macro for iterating over subblocks using the [traversal](https://crates.io/crates/traversal) crate.

# Vmf Format

Read more about the vmf format on [Valve Developer Community](https://developer.valvesoftware.com/wiki/Valve_Map_Format)

```vmf
// This is a comment.
ClassName_1
{
	"Property_1" "Value_1"
	"Property_2" "Value_2"
	ClassName_2
	{
		"Property_1" "Value_1"
	}
	ClassName_3
	{
	}
}
```

# Example

```rust
use vmf_parser_nom::ast::{Block};
use vmf_parser_nom::parse;
use vmf_parser_nom::{VerboseError, SimpleError, ErrorKind};

let input = "ClassName_1
{
\t\"Property_1\" \"Value_1\"
\t\"Property_2\" \"Value_2\"
\tClassName_2
\t{
\t\t\"Property_1\" \"Value_1\"
\t}
\tClassName_3
\t{
\t}
}";

// parse the input to a vmf, borrowing from input
let vmf = parse::<&str, ()>(input).unwrap();
let string = vmf.to_string();
println!("vmf {vmf}")
assert_eq!(input, string);

// parse to owned strings instead
let vmf_owned = parse::<String, ()>(input).unwrap();

// All valid error types
let invalid_input = "block{\"property_with_no_value\"}";
let err_verbose = parse::<&str, VerboseError<_>>(invalid_input).unwrap_err();
let err_simple = parse::<&str, SimpleError<_>>(invalid_input).unwrap_err();
let err_tuple = parse::<&str, (_, ErrorKind)>(invalid_input).unwrap_err();
let err_unit = parse::<&str, ()>(invalid_input).unwrap_err();

println!("verbose: {err_verbose:?}");
println!("simple: {err_simple:?}");
println!("tuple: {err_tuple:?}");
println!("unit: {err_unit:?}");

// implements Deref
let block: &Block<String> = &vmf_owned;
assert_eq!(vmf_owned.inner, *block);

// inner value is simply a block with no properties
assert_eq!(vmf_owned.inner.name, "root");
assert_eq!(vmf_owned.inner.props, vec![]);
assert!(!vmf_owned.inner.blocks.is_empty());
```