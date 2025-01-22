//! Generator for ID getter/setter methods on all types with `scope_id`, `symbol_id`, `reference_id`
//! fields.
//!
//! e.g. Generates `scope_id` and `set_scope_id` methods on all types with a `scope_id` field.

use quote::quote;

use crate::{
    output::{output_path, Output},
    schema::Schema,
    Generator,
};

use super::define_generator;

pub struct GetIdGenerator;

define_generator!(GetIdGenerator);

impl Generator for GetIdGenerator {
    fn generate(&self, _schema: &Schema) -> Output {
        Output::Rust { path: output_path(crate::AST_CRATE, "get_id.rs"), tokens: quote!() }
    }
}
