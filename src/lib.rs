#![allow(dead_code)]
#![warn(rustdoc::private_intra_doc_links)]
//! A parser for the Valve map format.
//! Also a provided convience [macro](crate::traverse) for iterating over subblocks using the [traversal](https://crates.io/crates/traversal) crate.
//!
//! # Vmf Format
//! 
//! See [`parse()`] for the implementation.
//! Read more about the vmf format on [Valve Developer Community](https://developer.valvesoftware.com/wiki/Valve_Map_Format)
//! 
//! ```vmf
#![doc = "// This is a comment.
ClassName_1
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
}"]
//! ```
//! 
//! # Example
//! 
//! ```rust
//! use vmf_parser_nom::ast::{Block};
//! use vmf_parser_nom::parse;
//! use vmf_parser_nom::{VerboseError, SimpleError, ErrorKind};
//! 
//! let input = "ClassName_1
//! {
//! \t\"Property_1\" \"Value_1\"
//! \t\"Property_2\" \"Value_2\"
//! \tClassName_2
//! \t{
//! \t\t\"Property_1\" \"Value_1\"
//! \t}
//! \tClassName_3
//! \t{
//! \t}
//! }";
//! 
//! // parse the input to a vmf, borrowing from input
//! let vmf = parse::<&str, ()>(input).unwrap();
//! let string = vmf.to_string();
//! println!("vmf {vmf}")
//! assert_eq!(input, string);
//! 
//! // parse to owned strings instead
//! let vmf_owned = parse::<String, ()>(input).unwrap();
//! 
//! // All valid error types
//! let invalid_input = "block{\"property_with_no_value\"}";
//! let err_verbose = parse::<&str, VerboseError<_>>(invalid_input).unwrap_err();
//! let err_simple = parse::<&str, SimpleError<_>>(invalid_input).unwrap_err();
//! let err_tuple = parse::<&str, (_, ErrorKind)>(invalid_input).unwrap_err();
//! let err_unit = parse::<&str, ()>(invalid_input).unwrap_err();
//! 
//! println!("verbose: {err_verbose:?}");
//! println!("simple: {err_simple:?}");
//! println!("tuple: {err_tuple:?}");
//! println!("unit: {err_unit:?}");
//! 
//! // implements Deref
//! let block: &Block<String> = &vmf_owned;
//! assert_eq!(vmf_owned.inner, *block);
//! 
//! // inner value is simply a block with no properties
//! assert_eq!(vmf_owned.inner.name, "root");
//! assert_eq!(vmf_owned.inner.props, vec![]);
//! assert!(!vmf_owned.inner.blocks.is_empty());
//! ```

mod owned;
pub use owned::*;

// dumb workaround for doc comments not interpreting \n
// and re-exports appending original documentation for some reason
#[doc = "Re-export of [`nom::error::Error`] for conveinience\n\n"]
pub use nom::error::Error as SimpleError;
#[doc = "Re-export of [`nom::error::ErrorKind`] for conveinience\n\n"]
pub use nom::error::ErrorKind;
#[doc = "Re-export of [`nom::error::VerboseError`] for conveinience\n\n"]
pub use nom::error::VerboseError;

use owned::ast::*;
use owned::parsers::nom_prelude::*;
use owned::parsers::vmf;

// pub(crate) type VerboseError<I> = VerboseError<I>;

/// Macro for making an iterator over all the children of a block using
/// the [traversal](https://docs.rs/traversal/latest/traversal/index.html) crate.
/// Calls `.as_ref()` on input so works for [`Vmf`]s and [`Block`]s.
/// Usage is `traverse!(<traverse struct>, block)`.
///
/// Valid traverse structs are:
/// [`traversal::Bft`](https://docs.rs/traversal/latest/traversal/struct.Bft.html), [`traversal::DftPre`](https://docs.rs/traversal/latest/traversal/struct.DftPre.html), [`traversal::DftPost`](https://docs.rs/traversal/latest/traversal/struct.DftPost.html), [`traversal::DftPreRev`](https://docs.rs/traversal/latest/traversal/struct.DftPreRev.html), [`traversal::DftPostRev`](https://docs.rs/traversal/latest/traversal/struct.DftPostRev.html).
///
/// These also work but return paths:
/// [`traversal::DftPaths`](https://docs.rs/traversal/latest/traversal/struct.DftPaths.html), [`traversal::DftLongestPaths`](https://docs.rs/traversal/latest/traversal/struct.DftLongestPaths.html), [`traversal::DftCycles`](https://docs.rs/traversal/latest/traversal/struct.DftCycles.html).
/// 
/// # Examples
/// 
/// ```rust
/// use traversal::Bft;
/// use vmf_parser_nom::traverse;
/// use vmf_parser_nom::parse;
/// 
/// let input = "block1{}block2{}block3{}";
/// let vmf = parse::<&str, ()>(input).unwrap();
/// 
/// // returns an iterator
/// let bft = traverse!(Bft, vmf);
/// for (level, block) in bft {
///     // prints:
///     // root @ level 0
///     // block1 @ level 1
///     // block2 @ level 1
///     // block3 @ level 1
///     println!("{} @ level {}", block.name, level);
/// }
/// ```
#[macro_export]
macro_rules! traverse {
    ($traverse_struct: ident, $block: expr) => {
        $traverse_struct::new($block.as_ref(), $crate::ast::Block::iter_children)
    };
}



// FromStr unable to be implemented because dumb lifetime stuff
/// Parse a `&str` into a [`Vmf`], completely ignoring whitespace.
/// You can specify the output string type to be
/// any type that implements `From<&str>`.
///
/// Valid error types are
/// `()`, [`(I, nom::error::ErrorKind)`](nom::error::ErrorKind), [`nom::error::Error<&str>`], [`nom::error::VerboseError<&str>`].
/// Or other types that impl [`ParseError`] and [`ContextError`]
/// 
/// See [Vmf Format](./index.html#vmf-format).
pub fn parse<'a, O, E>(input: &'a str) -> Result<Vmf<O>, E>
where
    O: From<&'a str>,
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    match vmf(input) {
        Ok((_, vmf)) => Ok(vmf),
        Err(e) => match e {
            nom::Err::Incomplete(_) => Err(ContextError::add_context(
                input,
                "incomplete",
                ParseError::from_error_kind(input, ErrorKind::Fail),
            )),
            nom::Err::Error(e) => Err(e),
            nom::Err::Failure(e) => Err(e),
        },
    }
}
