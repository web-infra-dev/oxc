use crate::{output::Output, schema::Schema, Generator};

use super::{attr_positions, define_generator, AttrPositions};

pub struct TypescriptGenerator;

define_generator!(TypescriptGenerator);

impl Generator for TypescriptGenerator {
    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[("ts", attr_positions!(StructField | EnumVariant))]
    }

    fn generate(&self, _schema: &Schema) -> Output {
        Output::Javascript {
            path: format!("{}/types.d.ts", crate::TYPESCRIPT_PACKAGE),
            code: String::new(),
        }
    }
}
