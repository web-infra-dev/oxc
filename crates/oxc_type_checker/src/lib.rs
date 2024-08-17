mod ast;
mod checker;
mod type_builder;
mod type_table;
mod sync;

pub(crate) use type_builder::TypeBuilder;
pub use type_table::TypeTable;
pub use checker::Checker;
