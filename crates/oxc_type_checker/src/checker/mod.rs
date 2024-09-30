mod composite;
mod get_type;
mod type_factory;
mod type_inquisition;

use std::{cell::Ref, rc::Rc};

use oxc_allocator::Allocator;
#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_ast::Visit;
use oxc_semantic::Semantic;
use oxc_syntax::types::{TypeFlags, TypeId};

use crate::{
    ast::Type,
    subsystem::{Intrinsics, TypeBuilder, TypeCache},
    CheckerSettings,
};

/// ## References
/// - <https://gist.github.com/Boshen/d189de0fe0720a30c5182cb666e3e9a5>
pub struct Checker<'a> {
    settings: CheckerSettings,
    builder: TypeBuilder<'a>,
    intrinsics: Intrinsics,
    semantic: Rc<Semantic<'a>>,
    cache: TypeCache<'a>,
}

// public interface
impl<'a> Checker<'a> {
    pub fn new(alloc: &'a Allocator, semantic: Rc<Semantic<'a>>) -> Self {
        let settings = CheckerSettings::default();
        let builder = TypeBuilder::new(alloc);
        let intrinsics = Intrinsics::new(&builder, &settings);
        let cache = TypeCache::new(alloc);

        Self { settings, builder, intrinsics, semantic, cache }
    }
}

// crate-public getters
impl<'a> Checker<'a> {
    pub(crate) fn get_flags(&self, type_id: TypeId) -> TypeFlags {
        self.builder.table().get_flags(type_id)
    }

    pub(crate) fn get_type(&self, type_id: TypeId) -> Ref<'_, Type<'a>> {
        Ref::map(self.builder.table(), |table| table.get_type(type_id))
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
