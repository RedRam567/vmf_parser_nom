//! Parse Vmf from a str

pub(crate) mod nom_prelude {
    pub use nom::{
        branch::alt,
        bytes::complete::{is_not, tag, take_until, take_while},
        character::complete::{
            alphanumeric0, alphanumeric1, char, multispace0, multispace1, one_of,
        },
        combinator::{fail, map, map_opt, map_res, opt, recognize, success, value},
        error::{context, ContextError, ErrorKind, ParseError, VerboseError, VerboseErrorKind},
        multi::{many0, many0_count, many1, many1_count},
        sequence::{pair, preceded, separated_pair, terminated, tuple},
        IResult, InputIter, InputLength, InputTake, InputTakeAtPosition, Parser,
    };
}

use crate::owned::ast::{Block, Property, Vmf};
use nom_prelude::*;

/// Parses a [`Vmf`]. Discards any whitespace.
pub fn vmf<'a, O, E>(input: &'a str) -> IResult<&'a str, Vmf<O>, E>
where
    O: From<&'a str>,
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    map(many1(block), Vmf::new)(input)
}

/// Parses a [`Block`]. Discards any whitespace.
pub fn block<'a, O, E>(input: &'a str) -> IResult<&'a str, Block<O>, E>
where
    O: From<&'a str>,
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    let (input, _) = many0_count(ignorable)(input)?;
    let (input, name) = terminated(ignore_whitespace(identifier), open_brace)(input)?;

    let mut props = Vec::new();
    let mut blocks = Vec::new();

    // let (input, _) = many0(alt((comment, add_prop, add_block)))(input)?;

    // manual `alt` implementation to allow break or pushing or smth
    let mut input = input;
    let mut has_ending_brace = false;
    // while !input.is_empty() {
    loop {
        // ugly loop
        if let Ok((i, prop)) = property::<_, E>(input) {
            props.push(prop);
            input = i;
        } else if let Ok((i, block)) = block::<_, E>(input) {
            blocks.push(block);
            input = i;
        } else if let Ok((i, ())) = ignorable::<E>(input) {
            input = i;
        } else if let Ok((i, _)) = ignore_whitespace(char::<_, E>('}'))(input) {
            input = i;
            has_ending_brace = true;
            break;
        } else if input.is_empty() {
            // needed for some reason, cant use if guard
            break;
        } else {
            // nom moment
            return Err(nom::Err::Error(ContextError::add_context(
                input,
                "no parsers matched in block",
                ParseError::from_error_kind(input, ErrorKind::Fail),
            )));
        }
    }

    if has_ending_brace {
        Ok((input, Block { name: name.into(), props, blocks }))
    } else {
        Err(nom::Err::Error(ContextError::add_context(
            input,
            "missing } in block",
            ParseError::from_error_kind(input, ErrorKind::Fail),
        )))
    }
}

// Parses a [`Property`] value in the form `\s"TEXT"\s"TEXT"\s`. Where `\s` zero or more whitespace according to [`multispace0`].
/// Parses a [`Property`]. Discards any whitespace.
pub fn property<'a, O, E>(input: &'a str) -> IResult<&'a str, Property<O, O>, E>
where
    O: From<&'a str>,
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    // let input: &'a str = input.as_ref();
    context(
        "property error",
        map(ignore_whitespace(separated_pair(string, multispace0, string)), |(key, value)| {
            Property { key: key.into(), value: value.into() }
        }),
    )(input)
    // )(input.as_ref())
}

/// Parses a string in the form: `"TEXT"`, TEXT is any character other than a double quote. Consumes double quotes, does not consume whitespace.
pub fn string<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    // map(tuple((char('"'), take_until("\""), char('"'))), |x| x.1)(input)
    context("string error", surrounded_by(char('"'), take_until("\""), char('"')))(input)
}

/// [`comment`] or [`multispace1`]
fn ignorable<'a, E>(input: &'a str) -> IResult<&'a str, (), E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    // value is like map but outputs new parser instead of modifying output directly
    context("ignorable error", alt((comment, value((), multispace1))))(input)
}

/// [`nom`] Parser for a comment in the form: `//TEXT\n`. Consumes whitespace, returns ()
pub fn comment<'a, E>(input: &'a str) -> IResult<&'a str, (), E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    context("comment error", value((), pair(tag("//"), is_not_no_fail("\n\r"))))(input)
}

/// "\s{\s"
fn open_brace<'a, E>(input: &'a str) -> IResult<&'a str, (), E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    context("missing '{'", value((), ignore_whitespace(char('{'))))(input)
}

/// "\s{\s"
fn close_brace<'a, E>(input: &'a str) -> IResult<&'a str, (), E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    context("missing '}'", value((), ignore_whitespace(char('}'))))(input)
}

// 0-9, a-z, A-Z, _
/// Parser for an identifier in the form: `Text_123`, identifier is one or more alphanumeric characters or an underscore.
/// Does not consume whitespace.
pub fn identifier<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    context("bad identifier", recognize(many1_count(alt((alphanumeric1, tag("_"))))))(input)
}

/// Matches first parser and discards its outout, matches second parser, matches third parser and discards its output.
/// Like the opposite of [`separated_pair`].
const fn surrounded_by<I, O1, O2, O3, E, F, G, H>(
    mut first: F,
    mut second: G,
    mut third: H,
) -> impl FnMut(I) -> IResult<I, O2, E>
where
    F: Parser<I, O1, E>,
    G: Parser<I, O2, E>,
    H: Parser<I, O3, E>,
{
    move |input: I| {
        let (input, _) = first.parse(input)?;
        let (input, output) = second.parse(input)?;
        let (input, _) = third.parse(input)?;
        Ok((input, output))
    }
}

// TODO: clean up bounds
/// Discards leading whitespace according to [`multispace0`], matches the parser, discards trailing whitespace.
const fn ignore_whitespace<I, O, E, F>(mut second: F) -> impl FnMut(I) -> IResult<I, O, E>
where
    I: Clone + InputTakeAtPosition,
    E: ParseError<I>,
    F: Parser<I, O, E>,
    <I as InputTakeAtPosition>::Item: nom::AsChar + Clone,
{
    move |input: I| {
        let (input, _) = multispace0.parse(input)?;
        let (input, output) = second.parse(input)?;
        let (input, _) = multispace0.parse(input)?;
        Ok((input, output))
    }
}

/// The same as [`is_not`] but doesn't fail if no chars before a matched one
/// because thats kinda dumb.
const fn is_not_no_fail<T, Input, Error: ParseError<Input>>(
    arr: T,
) -> impl Fn(Input) -> IResult<Input, Input, Error>
where
    Input: InputTakeAtPosition,
    T: nom::FindToken<<Input as InputTakeAtPosition>::Item>,
{
    move |i: Input| i.split_at_position_complete(|c| arr.find_token(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = "ClassName_1
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
    const INPUT_NO_WHITE: &str = "ClassName_1{\"Property_1\"\"Value_1\"\"Property_2\"\"Value_2\"ClassName_2{\"Property_1\"\"Value_1\"}ClassName_3{}}";

    #[test]
    fn block_test() {
        assert!(block::<&str, VerboseError<_>>("{").is_err());
        assert!(block::<&str, VerboseError<_>>("}").is_err());
        assert!(block::<&str, VerboseError<_>>("").is_err());
        let input = r#"
    // This is a comment.
    //

ClassName_1 {
        "Property_1"

  "Value_1"

        "Property_2""Value_2"""""
        ClassName_2
        {
            "Property_1" "Value_1"
        }
        ClassName_3{}
            //uh accepts missing closing brace thats kinda bad
                    
            //another comment, preceded by tabs
 }     
        
                "#;
        let truth = Block::new(
            "ClassName_1",
            vec![
                Property::new("Property_1", "Value_1"),
                Property::new("Property_2", "Value_2"),
                Property::new("", ""),
            ],
            vec![
                Block::new("ClassName_2", vec![Property::new("Property_1", "Value_1")], vec![]),
                Block::new("ClassName_3", vec![], vec![]),
            ],
        );
        let (i, output) = super::block::<&str, VerboseError<_>>(input).unwrap();
        eprintln!("{output}");
        eprintln!("result input {i:?}");
        assert_eq!(truth, output);
        assert!(i.is_empty());
    }

    #[test]
    fn prop() {
        let input = r#"        "Property_1" "Value_1"
        "Property_2" "Value_2"
        ClassName_2
        {
            "Property_1" "Value_1"
        }
        ClassName_3
        {
        }"#;
        let truth = Property::new("Property_1", "Value_1");
        let (i, output) = property::<&str, VerboseError<_>>(input).unwrap();
        eprintln!("result input {i:?}");
        assert_eq!(truth, output);
        assert!(!i.is_empty());
    }

    #[test]
    fn display() {
        let vmf = crate::parse::<&str, VerboseError<_>>(INPUT).unwrap();
        let output = vmf.to_string();

        let vmf_no_white = crate::parse::<&str, VerboseError<_>>(INPUT_NO_WHITE).unwrap();
        let output_no_white = vmf_no_white.to_string();

        assert_eq!(INPUT, output);
        assert_eq!(INPUT, output_no_white);
        assert_eq!(output, output_no_white);
    }
}
