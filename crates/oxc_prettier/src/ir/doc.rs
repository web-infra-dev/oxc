//! Prettier IR
//!
//! References:
//! * <https://github.com/prettier/prettier/blob/3.4.1/commands.md>

use oxc_allocator::{Box, Vec};

use crate::GroupId;

#[derive(Debug)]
pub enum Doc<'a> {
    Str(&'a str),
    // PERF: can we use &[Doc] here?
    Array(Vec<'a, Doc<'a>>),
    /// Increase the level of indentation.
    Indent(Vec<'a, Doc<'a>>),
    IndentIfBreak(IndentIfBreak<'a>),
    /// Mark a group of items which the printer should try to fit on one line.
    /// This is the basic command to tell the printer when to break.
    /// Groups are usually nested, and the printer will try to fit everything on one line,
    /// but if it doesn't fit it will break the outermost group first and try again.
    /// It will continue breaking groups until everything fits (or there are no more groups to break).
    Group(Group<'a>),
    /// Specify a line break.
    /// If an expression fits on one line, the line break will be replaced with a space.
    /// Line breaks always indent the next line with the current level of indentation.
    Line(Line),
    /// This is used to implement trailing comments.
    /// It's not practical to constantly check where the line ends to avoid accidentally printing some code at the end of a comment.
    /// `lineSuffix` buffers docs passed to it and flushes them before any new line.
    LineSuffix(Vec<'a, Doc<'a>>),
    /// Print something if the current `group` or the current element of `fill` breaks and something else if it doesn't.
    IfBreak(IfBreak<'a>),
    /// This is an alternative type of group which behaves like text layout:
    /// it's going to add a break whenever the next element doesn't fit in the line anymore.
    /// The difference with `group` is that it's not going to break all the separators, just the ones that are at the end of lines.
    Fill(Fill<'a>),
    /// Include this anywhere to force all parent groups to break.
    BreakParent,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Line {
    pub hard: bool,
    pub soft: bool,
    pub literal: bool,
}

#[derive(Debug)]
pub struct Group<'a> {
    // TODO: Vec? Box?
    pub contents: Vec<'a, Doc<'a>>,
    pub should_break: bool,
    pub expanded_states: Option<Vec<'a, Doc<'a>>>,
    #[allow(clippy::struct_field_names)]
    pub group_id: Option<GroupId>,
}

#[derive(Debug)]
pub struct IndentIfBreak<'a> {
    pub contents: Vec<'a, Doc<'a>>,
    pub group_id: Option<GroupId>,
}

#[derive(Debug)]
pub struct Fill<'a> {
    pub contents: Vec<'a, Doc<'a>>,
}

// Printer utils
impl<'a> Fill<'a> {
    pub fn drain_out_pair(&mut self) -> (Option<Doc<'a>>, Option<Doc<'a>>) {
        let content = if self.contents.len() > 0 { Some(self.contents.remove(0)) } else { None };
        let whitespace = if self.contents.len() > 0 { Some(self.contents.remove(0)) } else { None };
        (content, whitespace)
    }

    pub fn dequeue(&mut self) -> Option<Doc<'a>> {
        if self.contents.len() > 0 {
            Some(self.contents.remove(0))
        } else {
            None
        }
    }

    pub fn enqueue(&mut self, doc: Doc<'a>) {
        self.contents.insert(0, doc);
    }

    pub fn parts(&self) -> &[Doc<'a>] {
        &self.contents
    }

    pub fn take_parts(self) -> Vec<'a, Doc<'a>> {
        self.contents
    }
}

#[derive(Debug)]
pub struct IfBreak<'a> {
    pub break_contents: Box<'a, Doc<'a>>,
    pub flat_contents: Box<'a, Doc<'a>>,
    pub group_id: Option<GroupId>,
}
