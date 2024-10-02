//! `check*` methods (e.g. `checkExpression`, `checkSourceFile`) and related
//! flags/structs.

mod expression;
mod jsx;

use bitflags::bitflags;
use oxc_ast::ast::Expression;
use oxc_syntax::types::TypeId;
use std::cell::Cell;

use super::Checker;

// Public Checker API

impl<'a> Checker<'a> {
    #[inline]
    pub fn check_expression(&mut self, expr: &Expression<'a>) -> TypeId {
        expr.check(self, &CheckContext::default())
    }

    #[inline]
    pub fn check_expression_with_options(
        &mut self,
        expr: &Expression<'a>,
        check_mode: CheckMode,
        force_tuple: bool,
    ) -> TypeId {
        let ctx = CheckContext { mode: check_mode, force_tuple, ..Default::default() };
        expr.check(self, &ctx)
    }
}

// Check trait and stuff related to it

bitflags! {
    // src/compiler/checker.ts, line 1323
    /// TODO: impl ord?
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CheckMode: u8 {
        /// Normal type checking
        const Normal = 0;
        /// Explicitly assigned contextual type, therefore not cacheable
        const Contextual = 1 << 0;
        /// Inferential typing
        const Inferential = 1 << 1;
        /// Skip context-sensitive function expressions
        const SkipContextSensitive = 1 << 2;
        /// Skip single signature generic functions
        const SkipGenericFunctions = 1 << 3;
        /// Call resolution for purposes of signature help
        const IsForSignatureHelp = 1 << 4;
        /// Checking a type that is going to be used to determine the type of a rest binding element
        /// e.g. in `const { a, ...rest } = foo`, when checking the type of `foo` to determine the type of `rest`,
        /// we need to preserve generic types instead of substituting them for constraints
        const RestBindingElement = 1 << 5;
        /// Called from getTypeOfExpression, diagnostics may be omitted
        const TypeOnly = 1 << 6;
    }
}

impl Default for CheckMode {
    #[inline]
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Default, Clone /* intentionally not copy */)]
#[non_exhaustive]
pub(crate) struct CheckContext {
    /// Type checking mode.
    ///
    /// Note: in TypeScript, `checkMode` is `CheckMode | undefined`. This may
    /// become relevant; I'm not sure.
    ///
    /// Default: [`CheckMode::Normal`].
    mode: CheckMode,
    /// Force tuple types. Used when checking array expressions.
    ///
    /// Default: `false`
    force_tuple: bool,
    // todo: instantiationCount, instantiationDepth for depth limit checking in
    // `instantiateTypeWithAlias`
    instantiation_count: Cell<usize>,
    instantiation_depth: Cell<usize>,
}

pub(crate) trait Check<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId;
}
