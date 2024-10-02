//! `check*` for JSX nodes

#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_syntax::types::TypeId;

use super::{Check, CheckContext, Checker};

// TODO: SyntheticExpression
// TODO: JsxExpression
// NOTE:
impl<'a> Check<'a> for JSXElement<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkJsxElement")
    }
}
impl<'a> Check<'a> for JSXFragment<'a> {
    fn check(
        &self,
        checker: &mut Checker<'a>,
        ctx: &super::CheckContext,
    ) -> oxc_syntax::types::TypeId {
        todo!("checkJsxFragment")
    }
}
