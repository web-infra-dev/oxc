use std::marker::PhantomData;

use oxc_ast::ast::*;

use crate::{
    context::TransformerCtx,
    options::{TransformOptions, TransformTarget},
};

/// ES2021: Numeric Separator
///
/// References:
/// * <https://babeljs.io/docs/babel-plugin-transform-numeric-separator>
pub struct NumericSeparator<'a>(PhantomData<&'a ()>);

impl<'a> NumericSeparator<'a> {
    #![allow(clippy::unused_self)]

    pub fn new(_: TransformerCtx<'a>, options: &TransformOptions) -> Option<Self> {
        (options.target < TransformTarget::ES2021 || options.numeric_separator)
            .then_some(Self(PhantomData {}))
    }

    pub fn transform_number_literal(&mut self, lit: &mut NumericLiteral<'a>) {
        if !lit.raw.is_empty() {
            // set literal raw string to empty so codegen have to use the value.
            lit.raw = "";
        }
    }

    pub fn transform_bigint_literal(&mut self, lit: &mut BigIntLiteral<'a>) {
        let raw = &lit.raw;
        if !raw.is_empty() && raw.contains('_') {
            lit.raw = raw.replace('_', "").into();
        }
    }
}
