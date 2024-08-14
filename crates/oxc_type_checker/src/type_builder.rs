#[allow(clippy::wildcard_imports)]
use crate::ast::*;
use std::{
    cell::{Ref, RefCell, RefMut},
    fmt,
    rc::Rc,
};

use oxc_allocator::{Allocator, Box};
use oxc_syntax::types::{ObjectFlags, TypeFlags, TypeId};

use crate::TypeTable;

#[derive(Clone)]
pub(crate) struct TypeBuilder<'a> {
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

    pub fn table(&self) -> Ref<'_, TypeTable<'a>> {
        self.table.as_ref().borrow()
    }

    fn table_mut(&self) -> RefMut<'_, TypeTable<'a>> {
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
        self.table_mut().create_type(ty, flags, None, None)
    }

    // /// Creates a [`UnionType`]
    // pub fn create_union_type(
    //     &self,
    //     types: &[TypeId],
    //     union_reduction: UnionReduction,
    //     alias_symbol: Option<SymbolId>,
    //     type_alias_arguments: Option<&[TypeId]>,
    //     origin: Option<TypeId>,
    // ) -> TypeId {
    //     match types.len() {
    //         0 => self.table.
    //     }
    // }
}

// #[derive(Debug, Default, Clone, Copy, PartialEq, Ord)]
// pub enum UnionReduction {
//     None = 0,
//     #[default]
//     Literal,
//     Subtype,
// }

impl fmt::Debug for TypeBuilder<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypeBuilder").field("table", &self.table).finish()
    }
}
