mod get_type;
mod intrinsics;
mod settings;

use std::rc::Rc;

use oxc_allocator::Allocator;
#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_ast::Visit;
use oxc_semantic::Semantic;
use oxc_syntax::types::{TypeFlags, TypeId};

use crate::TypeBuilder;
use intrinsics::Intrinsics;
use settings::CheckerSettings;

/// ## References
/// - <https://gist.github.com/Boshen/d189de0fe0720a30c5182cb666e3e9a5>
pub struct Checker<'a> {
    settings: CheckerSettings,
    builder: TypeBuilder<'a>,
    intrinsics: Intrinsics,
    semantic: Rc<Semantic<'a>>,
}

impl<'a> Checker<'a> {
    pub fn new(alloc: &'a Allocator, semantic: Rc<Semantic<'a>>) -> Self {
        let settings = CheckerSettings::default();
        let builder = TypeBuilder::new(alloc);
        let intrinsics = Intrinsics::new(&builder, &settings);

        Self { settings, builder, intrinsics, semantic }
    }
}

impl<'a> Visit<'a> for Checker<'a> {
    fn visit_ts_type_alias_declaration(&mut self, it: &oxc_ast::ast::TSTypeAliasDeclaration<'a>) {}
}

pub(crate) trait Check<'a> {
    fn check(&self, checker: &mut Checker<'a>) -> TypeId;
}

impl<'a> Check<'a> for TSTypeAliasDeclaration<'a> {
    fn check(&self, checker: &mut Checker<'a>) -> TypeId {
        todo!()
    }
}
