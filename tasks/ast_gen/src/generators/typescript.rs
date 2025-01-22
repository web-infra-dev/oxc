use crate::{output::Output, schema::Schema, Generator};

use super::define_generator;

pub struct TypescriptGenerator;

define_generator!(TypescriptGenerator);

impl Generator for TypescriptGenerator {
    fn field_attrs(&self) -> &[&'static str] {
        // TODO: This generator doesn't actually use this attr. Nothing does at present.
        &["ts"]
    }

    fn variant_attrs(&self) -> &[&'static str] {
        // TODO: This generator doesn't actually use this attr. Nothing does at present.
        &["ts"]
    }

    fn generate(&self, _schema: &Schema) -> Output {
        Output::Javascript {
            path: format!("{}/types.d.ts", crate::TYPESCRIPT_PACKAGE),
            code: String::new(),
        }
    }
}
