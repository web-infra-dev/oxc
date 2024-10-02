mod ast_impl;
mod literal;
mod object;
mod signature;

use oxc_allocator::{Box, Vec};
use oxc_syntax::types::{ObjectFlags, TypeId};

pub use self::{literal::*, object::*};

#[derive(Debug)]
pub enum Type<'a> {
    // Mostly done. missing freshability
    Literal(Box<'a, LiteralType<'a>>),
    // complete
    Intrinsic(Box<'a, IntrinsicType<'a>>),
    // complete?
    FreshableIntrinsic(Box<'a, FreshableIntrinsicType<'a>>),
    // 70% done. Missing a lot of union reduction logic
    Union(Box<'a, UnionType<'a>>),
    // barely started
    Object(Box<'a, ObjectType<'a>>),
}

pub type FreshableIntrinsicType<'a> = FreshableType<IntrinsicType<'a>>;

#[derive(Debug)]
pub struct IntrinsicType<'a> {
    pub name: &'a str,
    // TODO: optimize size by removing debug_name in release builds?
    // #[cfg(debug_assertions)]
    pub(crate) debug_name: Option<&'a str>,
    pub object_flags: ObjectFlags,
}

#[derive(Debug)]
pub struct UnionType<'a> {
    pub types: Vec<'a, TypeId>,
    pub object_flags: ObjectFlags,
    /// Denormalized union, intersection, or index type in which union originates
    pub(crate) origin: Option<TypeId>, // TODO: add the other fields
}

/// `src/compiler/types.ts`, line 6399
///
/// ```typescript
/// export interface FreshableType extends Type {
///     freshType: FreshableType; // Fresh version of type
///     regularType: FreshableType; // Regular version of type
/// }
/// ```
#[derive(Debug)]
pub enum FreshableType<T> {
    /// Regular version of the type
    Regular(T, /* freshType */ TypeId),
    /// Fresh version of the type
    Fresh(T, /* regularType. None if fresh type is the regular type */ Option<TypeId>),
}
