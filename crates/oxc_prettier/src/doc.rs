//! Prettier IR
//!
//! References:
//! * <https://github.com/prettier/prettier/blob/main/commands.md>

use oxc_allocator::{Box, String, Vec};
use serde::Serialize;

use crate::Prettier;

#[derive(Debug, Serialize)]
pub enum Doc<'a> {
    Str(&'a str),
    // perf: can we use &[Doc] here?
    Array(Vec<'a, Doc<'a>>),
    /// Increase the level of indentation.
    Indent(Vec<'a, Doc<'a>>),
    /// Mark a group of items which the printer should try to fit on one line.
    /// This is the basic command to tell the printer when to break.
    /// Groups are usually nested, and the printer will try to fit everything on one line,
    /// but if it doesn't fit it will break the outermost group first and try again.
    /// It will continue breaking groups until everything fits (or there are no more groups to break).
    Group(Vec<'a, Doc<'a>>),
    /// Specify a line break.
    /// If an expression fits on one line, the line break will be replaced with a space.
    /// Line breaks always indent the next line with the current level of indentation.
    Line,
    /// Specify a line break.
    /// The difference from line is that if the expression fits on one line, it will be replaced with nothing.
    Softline,
    /// Specify a line break that is **always** included in the output,
    /// no matter if the expression fits on one line or not.
    Hardline,
    /// Print something if the current `group` or the current element of `fill` breaks and something else if it doesn't.
    IfBreak(Box<'a, Doc<'a>>),
}

#[derive(Clone, Copy)]
#[allow(unused)]
pub enum Separator {
    Softline,
    Hardline,
}

/// Doc Builder
impl<'a> Prettier<'a> {
    #[inline]
    pub(crate) fn vec<T>(&self) -> Vec<'a, T> {
        Vec::new_in(self.allocator)
    }

    #[inline]
    pub(crate) fn str(&self, s: &str) -> Doc<'a> {
        Doc::Str(String::from_str_in(s, self.allocator).into_bump_str())
    }

    #[inline]
    pub(crate) fn alloc(&self, doc: Doc<'a>) -> Box<'a, Doc<'a>> {
        Box(self.allocator.alloc(doc))
    }

    #[allow(unused)]
    pub(crate) fn join(
        &self,
        separator: Separator,
        docs: std::vec::Vec<Doc<'a>>,
    ) -> Vec<'a, Doc<'a>> {
        let mut parts = self.vec();
        for (i, doc) in docs.into_iter().enumerate() {
            if i != 0 {
                parts.push(match separator {
                    Separator::Softline => Doc::Softline,
                    Separator::Hardline => Doc::Hardline,
                });
            }
            parts.push(doc);
        }
        parts
    }
}
