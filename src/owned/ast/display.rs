use super::*;
use std::fmt::{self, Display, Write};

// TODO: dyn or impl/trait, both work. Can be nested PadAdapter or bare formatter
/// Helper struct for pretty printing struct like objects.
/// When nested, each adapter keeps track wether it should print padding.
/// See <https://github.com/rust-lang/rust/blob/master/library/core/src/fmt/builders.rs>
struct PadAdapter<'a> {
    buf: &'a mut dyn Write,
    on_newline: bool,
}

impl<'a> PadAdapter<'a> {
    fn new(buf: &'a mut dyn Write) -> Self {
        Self { buf, on_newline: false }
    }
}

impl fmt::Write for PadAdapter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for s in s.split_inclusive('\n') {
            if self.on_newline {
                self.buf.write_str(FMT_PADDING)?;
            }

            self.on_newline = s.ends_with('\n');
            self.buf.write_str(s)?;
        }

        Ok(())
    }
}

/// Stores the current max ids for [`Block::fmt_new_ids`]
/// Does not store/mess with visgroup ids or group ids as those are referenced
/// by the `Editor` info for entities
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct IdState {
    max_world_id: i32,
    max_solid_id: i32,
    max_side_id: i32,
    max_entity_id: i32,
}

impl<S: Display + AsRef<str>> Vmf<S> {
    /// Convert into a `String`. [`Display`] with alternate flag `{:#}` does the same thing.
    /// Generates new ids for solids, sides, entities, and worlds.
    /// Disregards any existing id (id can be omitted).
    pub fn to_string_new_ids(&self) -> String {
        format!("{self:#}")
    }
}

impl<S: Display + AsRef<str>> Block<S> {
    // TODO: dyn or impl, both work
    /// The [`Display`] alt implementation.
    /// Generates new ids for solids, sides, entities, and worlds.
    /// Disregards any existing id (id can be omitted).
    pub fn fmt_new_ids(&self, f: &mut dyn Write, state: &mut IdState) -> fmt::Result {
        writeln!(f, "{}", self.name)?;
        let mut adapter = PadAdapter::new(f);
        writeln!(adapter, "{{")?;

        self.write_new_id(&mut adapter, state)?;
        for prop in self.props.iter() {
            if !prop.is_id() {
                writeln!(adapter, "{prop}")?;
            }
        }

        for block in self.blocks.iter() {
            block.fmt_new_ids(&mut adapter, state)?;
            writeln!(&mut adapter)?;
        }

        write!(f, "}}")?;
        Ok(())
    }

    // TODO: dyn or impl, both work
    /// increment id and write.
    fn write_new_id(&self, f: &mut dyn Write, state: &mut IdState) -> fmt::Result {
        // ugly
        let new_id = match self.name.as_ref() {
            "world" => {
                state.max_world_id += 1;
                state.max_world_id
            }
            "solid" => {
                state.max_solid_id += 1;
                state.max_solid_id
            }
            "side" => {
                state.max_side_id += 1;
                state.max_side_id
            }
            "entity" => {
                state.max_entity_id += 1;
                state.max_entity_id
            }
            _ => {
                // ignore
                // #[cfg(debug_assertions)]
                // eprintln!("Unknown class `{}` with id property, ignoring", self.name);
                writeln!(f, "{self}")?;
                return Ok(());
            }
        };
        // reuse property display
        writeln!(f, "{}", Property::<&str, i32>::new("id", new_id))?;
        Ok(())
    }
}

impl<S: Display + AsRef<str>> Display for Vmf<S> {
    /// Formats the value using the given formatter. Alternate flag `{:#}` will
    /// generate new ids for solids, sides, entities, and worlds.
    /// Disregards any existing id (id can be omitted).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let is_alternate = f.alternate();
        let mut state = IdState::default();

        // too bad there isnt a better way to do see if end
        let mut iter = self.inner.blocks.iter().peekable();
        while let Some(block) = iter.next() {
            if is_alternate {
                block.fmt_new_ids(f, &mut state)?;
            } else {
                write!(f, "{block}")?;
            }
            if iter.peek().is_some() {
                // print newline if not last iteration
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl<S: Display> Display for Block<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.name)?;

        let mut adapter = PadAdapter::new(f);
        writeln!(adapter, "{{")?;
        for prop in self.props.iter() {
            writeln!(adapter, "{prop}")?;
        }
        for block in self.blocks.iter() {
            writeln!(adapter, "{block}")?;
        }

        write!(f, "}}")?;
        Ok(())
    }
}

impl<K: Display, V: Display> Display for Property<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\" \"{}\"", self.key, self.value)
    }
}

// most other parsing/display tests are in `parsers` module
#[cfg(test)]
mod tests {

    const INPUT_ID: &str = r#"world {}
world{ "id" "O_O two worlds incredibly rare/dumb but supported" }
solid { 
    "id" "not a number"
    side { "id" "42" }
    side { "id" "420" }
    side { "id" "69" }
}
solid { "id" "infinity" }
entity {}
entity { entity {} }
"#;

    #[test]
    fn alternate() {
        let truth_str = r#"world { "id" "1" }
world{ "id" "2" }
solid { 
    "id" "1"
    side { "id" "1" }
    side { "id" "2" }
    side { "id" "3" }
}
solid { "id" "2" }
entity { "id" "1" }
entity { "id" "2" entity { "id" "3" } }
"#;
        let truth = crate::parse::<&str, ()>(truth_str).unwrap();
        let input = crate::parse::<&str, ()>(INPUT_ID).unwrap();
        let output_str = format!("{input:#}");
        let output = crate::parse::<&str, ()>(&output_str).unwrap();

        eprintln!("{output_str}");
        assert_eq!(truth, output);
    }
}
