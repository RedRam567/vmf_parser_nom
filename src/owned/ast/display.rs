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

impl<S: Display> Vmf<S> {
    /// Used by the [`Display`] implementation
    pub fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // too bad there isnt a better way to do see if end
        let mut iter = self.inner.blocks.iter().peekable();
        while let Some(block) = iter.next() {
            if iter.peek().is_none() {
                // don't print newline on last iteration
                write!(f, "{block}")?;
            } else {
                writeln!(f, "{block}")?;
            }
        }
        Ok(())
    }

    /// Used by the [`Display`] implementation
    pub fn fmt_new_ids(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // too bad there isnt a better way to do see if end
        let mut iter = self.inner.blocks.iter().peekable();
        while let Some(block) = iter.next() {
            if iter.peek().is_none() {
                // don't print newline on last iteration
                write!(f, "{block}")?;
            } else {
                writeln!(f, "{block}")?;
            }
        }
        Ok(())
    }
}

impl<S: Display> Display for Vmf<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt(f)
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
