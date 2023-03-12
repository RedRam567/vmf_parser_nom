//! Abstract syntax tree representing a vmf file.

mod display;

pub use display::*;

use std::fmt::Write;
use std::ops::{Deref, DerefMut};

/// Padding for [`PadAdapter`]
const FMT_PADDING: &str = "\t";

/// A simple list of blocks, representing an enitre Vmf file. Implmented as a special block with a name
/// of [`Vmf::ROOT_NAME`] with no properties. `Vmf` implements [`Deref<Target = Block>`](Deref),
/// so all of [`Block`]s methods apply to `Vmf`.
///
/// See the [Vmf format](../../index.html#vmf-format).
#[repr(transparent)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Vmf<S> {
    pub inner: Block<S>,
}

/// A named block containing properties and other blocks.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Block<S> {
    pub name: S,
    // A vmf solid side has 8 properties and is extremely common.
    // Entities can have a widly varaible amount.
    pub props: Vec<Property<S, S>>,
    // 2 is same size as Vec, hmm often 6 sides tho, or 0-1 blocks
    pub blocks: Vec<Block<S>>,
}

/// A simple key-value pair.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Property<K, V> {
    pub key: K,
    pub value: V,
}

impl<S> Vmf<S> {
    pub const ROOT_NAME: &str = "root";

    /// Returns the root block. You can also use `.as_ref()` or Deref coercion.
    pub fn root(&self) -> &Block<S> {
        &self.inner
    }

    /// Returns the root block. You can also use `.as_mut()` or Deref coercion.
    pub fn root_mut(&mut self) -> &mut Block<S> {
        &mut self.inner
    }
}

impl<'a, S: From<&'a str>> Vmf<S> {
    pub fn new(blocks: Vec<Block<S>>) -> Self {
        Self { inner: Block::new(Self::ROOT_NAME, vec![], blocks) }
    }
}

impl<S> Block<S> {
    pub fn new<T: Into<S>>(name: T, props: Vec<Property<S, S>>, blocks: Vec<Block<S>>) -> Self {
        Self { name: name.into(), props, blocks }
    }

    /// Iterates over the sub blocks of this block. Not any of the children's children though.
    /// [`traverse`](crate::traverse) uses this. TODO:
    pub fn iter_children(&self) -> impl Iterator<Item = &Self> {
        self.blocks.iter()
    }
}

impl<S, V> Property<S, V> {
    pub fn new<T: Into<S>, U: Into<V>>(key: T, value: U) -> Self {
        Self { key: key.into(), value: value.into() }
    }
}

// Trait impls

impl<'a, S: From<&'a str>> Default for Vmf<S> {
    fn default() -> Self {
        Self { inner: Block::new(Self::ROOT_NAME, vec![], vec![]) }
    }
}

impl<S> AsRef<Block<S>> for Vmf<S> {
    fn as_ref(&self) -> &Block<S> {
        &self.inner
    }
}

impl<S> AsMut<Block<S>> for Vmf<S> {
    fn as_mut(&mut self) -> &mut Block<S> {
        &mut self.inner
    }
}

impl<S> Deref for Vmf<S> {
    type Target = Block<S>;

    fn deref(&self) -> &Block<S> {
        &self.inner
    }
}

impl<S> DerefMut for Vmf<S> {
    fn deref_mut(&mut self) -> &mut Block<S> {
        &mut self.inner
    }
}

impl<S> From<Vmf<S>> for Block<S> {
    fn from(vmf: Vmf<S>) -> Self {
        vmf.inner
    }
}
