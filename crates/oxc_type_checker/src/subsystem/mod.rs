//! Subsystems internal to [`Checker`] that, when composed, form the bulk of
//! Checker's structure.
#![allow(unused_imports)]

mod intrinsics;
mod links;
mod type_builder;
mod type_cache;
mod type_table;

pub(crate) use intrinsics::Intrinsics;
pub(crate) use links::{Links, SymbolLinks};
pub(crate) use type_builder::TypeBuilder;
pub(crate) use type_cache::{TypeCache, TypeList};
pub use type_table::TypeTable;
