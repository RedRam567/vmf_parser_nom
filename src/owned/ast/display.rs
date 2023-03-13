use super::*;
use std::fmt::{self, Display, Write};

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

/// Stores the current max ids for the TODO: ALTERNET DISP.
/// Does not store visgroup ids or group ids as those are referenced by the `Editor` info for entities
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdState {
    max_world_id: i32,
    max_solid_id: i32,
    max_side_id: i32,
    max_entity_id: i32,
}

impl Default for IdState {
    fn default() -> Self {
        Self { max_world_id: 1, max_solid_id: 1, max_side_id: 1, max_entity_id: 1 }
    }
}

impl<S: Display + AsRef<str>> Block<S> {
    /// Used by the [`Display`] alt implementation.
    fn fmt_new_ids(&self, f: &mut fmt::Formatter<'_>, state: &mut IdState) -> fmt::Result {
        writeln!(f, "{}", self.name)?;
        let is_alternate = f.alternate();
        let mut adapter = PadAdapter::new(f);
        writeln!(adapter, "{{")?;

        // props
        if is_alternate {
            self.write_new_id(&mut adapter, state)?;
        }
        for prop in self.props.iter() {
            if !is_alternate && prop.is_id() {
                writeln!(adapter, "{prop}")?;
            }
            writeln!(adapter, "{prop}")?;
        }

        // blocks
        for block in self.blocks.iter() {
            writeln!(adapter, "{block}")?;
        }

        write!(f, "}}")?;
        Ok(())
    }

    /// increment id and write.
    fn write_new_id(&self, f: &mut impl Write, state: &mut IdState) -> fmt::Result {
        // (ugly)
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
                #[cfg(debug_assertions)]
                eprintln!("Unknown class `{}` with id property, ignoring", self.name);
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
    /// So the id field can be ommited in that case.
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

impl<'a> fmt::Write for PadAdapter<'a> {
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
