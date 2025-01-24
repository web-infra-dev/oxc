use std::marker::PhantomData;
use std::ops::Range;

use oxc_index::{define_index_type, Idx, IndexVec};

define_index_type! {
    /// ID of type in the AST
    pub struct TypeId = u32;
}

define_index_type! {
    /// ID of source file
    pub struct FileId = u32;
}

/// Iterator over ID keys of an `IndexVec`.
pub struct IdIter<I: Idx> {
    range: Range<u32>,
    _marker: PhantomData<I>,
}

impl<I: Idx> IdIter<I> {
    /// Create `IdIter` iterator.
    #[expect(clippy::cast_possible_truncation)]
    pub(super) fn new<T>(vec: &IndexVec<I, T>) -> Self {
        Self { range: 0..vec.len() as u32, _marker: PhantomData }
    }
}

impl<I: Idx> Iterator for IdIter<I> {
    type Item = I;

    fn next(&mut self) -> Option<I> {
        self.range.next().map(|index| I::from_usize(index as usize))
    }
}
