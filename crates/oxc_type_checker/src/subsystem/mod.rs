//! Subsystems internal to [`Checker`] that, when composed, form the bulk of
//! Checker's structure.

mod intrinsics;
mod type_builder;
mod type_cache;
mod type_table;

pub(crate) use intrinsics::Intrinsics;
pub(crate) use type_builder::TypeBuilder;
pub(crate) use type_cache::TypeCache;
pub use type_table::TypeTable;
