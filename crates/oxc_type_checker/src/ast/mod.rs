mod ast_impl;
mod literal;
mod object;
mod signature;

use oxc_allocator::{Box, Vec};
use oxc_syntax::types::{ObjectFlags, TypeId};

pub use self::{literal::*, object::*};

#[derive(Debug)]
pub enum Type<'a> {
    Literal(Box<'a, LiteralType<'a>>),
    Intrinsic(Box<'a, IntrinsicType<'a>>),
    Union(Box<'a, UnionType<'a>>),
    Object(Box<'a, ObjectType<'a>>),
}

#[derive(Debug)]
pub struct IntrinsicType<'a> {
    pub name: &'a str,
    // TODO: optimize size by removing debug_name in release builds?
    // #[cfg(debug_assertions)]
    pub(crate) debug_name: Option<&'a str>,
    pub object_flags: ObjectFlags,
    // TODO: freshability
}

#[derive(Debug)]
pub struct UnionType<'a> {
    pub types: Vec<'a, TypeId>,
    pub object_flags: ObjectFlags,
    /// Denormalized union, intersection, or index type in which union originates
    pub(crate) origin: Option<TypeId>, // TODO: add the other fields
}
