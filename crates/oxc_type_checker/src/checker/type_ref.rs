use crate::ast::Type;
use oxc_syntax::types::TypeId;
use std::{ops::{Deref, DerefMut}, cell::{Ref, RefMut}};

use super::Checker;

#[derive(Clone, Copy)]
pub struct TypeRef<'a, 'c> {
    id: TypeId,
    checker: &'c Checker<'a>,
    // borrow: std::cell::Ref<>
}
impl<'a, 'c> TypeRef<'a, 'c> {
    fn new(id: TypeId, checker: &'c crate::Checker<'a>) -> Self {
        let table = checker.builder.table();
        // Ref::leak()
        Ref::undo_leak();
        todo!()
    }
}
impl<'a, 'c> Deref for TypeRef<'a, 'c> {
    // type Target = Ref<'c, Type<'a>>;
    type Target = Type<'a>;
    fn deref(&self) -> &Self::Target {
        todo!()
        self.checker.get_type(self.id).deref()
    }
}
