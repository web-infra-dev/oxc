mod literal;

pub use literal::*;
use oxc_allocator::{Box, Vec};
use oxc_syntax::types::{ObjectFlags, TypeId};

#[derive(Debug)]
pub enum Type<'a> {
    Literal(Box<'a, LiteralType<'a>>),
    Intrinsic(Box<'a, IntrinsicType<'a>>),
    Union(Box<'a, UnionType<'a>>),
}

#[derive(Debug)]
pub struct IntrinsicType<'a> {
    pub name: &'a str,
    // #[cfg(debug_assertions)]
    pub(crate) debug_name: Option<&'a str>, // TODO: freshability
    pub object_flags: ObjectFlags,
}

#[derive(Debug)]
pub struct UnionType<'a> {
    pub types: Vec<'a, TypeId>,
    pub object_flags: ObjectFlags,
    // TODO: add the other fields
}
