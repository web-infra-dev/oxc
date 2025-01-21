use quote::quote;

use crate::{
    output::{output_path, Output},
    schema::Schema,
    Generator,
};

use super::define_generator;

pub struct AstKindGenerator;

define_generator!(AstKindGenerator);

impl Generator for AstKindGenerator {
    fn generate(&mut self, _schema: &Schema) -> Output {
        Output::Rust { path: output_path(crate::AST_CRATE, "ast_kind.rs"), tokens: quote!() }
    }
}
