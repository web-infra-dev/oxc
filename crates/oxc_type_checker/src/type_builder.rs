#[allow(clippy::wildcard_imports)]
use crate::ast::*;
use std::{
    cell::{RefCell, RefMut},
    fmt,
    rc::Rc,
};

use oxc_allocator::{Allocator, Box};
use oxc_syntax::types::{ObjectFlags, TypeFlags, TypeId};
// use oxc_type_ast::Type;

use crate::TypeTable;

#[derive(Clone)]
pub struct TypeBuilder<'a> {
    alloc: &'a Allocator,
    table: Rc<RefCell<TypeTable<'a>>>,
}

impl<'a> TypeBuilder<'a> {
    pub fn new(alloc: &'a Allocator) -> Self {
        Self { alloc, table: Rc::new(RefCell::new(TypeTable::default())) }
    }

    fn alloc<T>(&self, value: T) -> Box<'a, T> {
        Box::new_in(value, self.alloc)
    }

    fn table(&self) -> RefMut<'_, TypeTable<'a>> {
        self.table.as_ref().borrow_mut()
    }

    /// Creates an [`IntrinsicType`]
    pub fn create_intrinsic_type(
        &self,
        flags: TypeFlags,
        name: &'a str,
        debug_name: Option<&'a str>,
    ) -> TypeId {
        self.create_intrinsic_object_type(flags, name, ObjectFlags::None, debug_name)
    }

    /// Creates an [`IntrinsicType`]
    pub fn create_intrinsic_object_type(
        &self,
        flags: TypeFlags,
        name: &'a str,
        object_flags: ObjectFlags,
        debug_name: Option<&'a str>,
    ) -> TypeId {
        let ty = Type::Intrinsic(self.alloc(IntrinsicType { name, debug_name, object_flags }));
        self.table().create_type(ty, flags, None, None)
    }
}

impl fmt::Debug for TypeBuilder<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypeBuilder").field("table", &self.table).finish()
    }
}
