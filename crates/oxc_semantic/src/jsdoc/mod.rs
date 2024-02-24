mod builder;

use std::{cell::OnceCell, collections::BTreeMap};

pub use builder::JSDocBuilder;
use oxc_span::{GetSpan, Span};

use self::parser::JSDocParser;
pub use self::parser::JSDocTag;
use crate::AstNode;

mod parser;

#[derive(Debug)]
pub struct JSDoc<'a> {
    /// JSDocs by Span
    attached: BTreeMap<Span, Vec<JSDocComment<'a>>>,
    not_attached: Vec<JSDocComment<'a>>,
}

#[derive(Debug, Clone)]
pub struct JSDocComment<'a> {
    comment: &'a str,
    /// Cached JSDocTags
    tags: OnceCell<Vec<JSDocTag<'a>>>,
}

impl<'a> JSDoc<'a> {
    pub fn new(
        attached: BTreeMap<Span, Vec<JSDocComment<'a>>>,
        not_attached: Vec<JSDocComment<'a>>,
    ) -> Self {
        Self { attached, not_attached }
    }

    pub fn get_by_node<'b>(&'b self, node: &AstNode<'a>) -> Option<Vec<JSDocComment<'a>>> {
        if !node.flags().has_jsdoc() {
            return None;
        }
        let span = node.kind().span();
        self.get_by_span(span)
    }

    pub fn get_by_span<'b>(&'b self, span: Span) -> Option<Vec<JSDocComment<'a>>> {
        self.attached.get(&span).cloned()
    }

    pub fn iter_all<'b>(&'b self) -> impl Iterator<Item = &JSDocComment<'a>> + 'b {
        self.attached.values().flatten().chain(self.not_attached.iter())
    }
}

impl<'a> JSDocComment<'a> {
    pub fn new(comment: &'a str) -> JSDocComment<'a> {
        Self { comment, tags: OnceCell::new() }
    }

    pub fn tags<'b>(&'b self) -> &'b Vec<JSDocTag<'a>> {
        self.tags.get_or_init(|| JSDocParser::new(self.comment).parse())
    }
}
