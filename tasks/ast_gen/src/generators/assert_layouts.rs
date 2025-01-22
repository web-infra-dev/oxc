use quote::quote;

use crate::{
    output::{output_path, Output},
    schema::Schema,
    Generator,
};

use super::define_generator;

pub struct AssertLayouts;

define_generator!(AssertLayouts);

impl Generator for AssertLayouts {
    fn generate(&self, _schema: &Schema) -> Output {
        Output::Rust { path: output_path(crate::AST_CRATE, "assert_layouts.rs"), tokens: quote!() }
    }
}
