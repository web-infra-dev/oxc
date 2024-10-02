#[allow(clippy::wildcard_imports)]
use crate::ast::*;
use std::{
    cell::{Ref, RefCell, RefMut},
    fmt,
    rc::Rc,
};

use oxc_allocator::{Allocator, Box, String, Vec};
use oxc_semantic::SymbolId;
use oxc_syntax::types::{ObjectFlags, TypeFlags, TypeId};

use super::TypeTable;

/// Creates new types and manages the memory arena.
#[derive(Clone)]
pub(crate) struct TypeBuilder<'a> {
    alloc: &'a Allocator,
    table: Rc<RefCell<TypeTable<'a>>>,
}

impl<'a> TypeBuilder<'a> {
    #[must_use]
    pub fn new(alloc: &'a Allocator) -> Self {
        Self { alloc, table: Rc::new(RefCell::new(TypeTable::default())) }
    }

    #[must_use]
    fn alloc<T>(&self, value: T) -> Box<'a, T> {
        Box::new_in(value, self.alloc)
    }

    /// Create a new empty [`Vec`] inside the builder's memory arena.
    #[must_use]
    pub fn vec<T>(&self) -> Vec<'a, T> {
        Vec::new_in(self.alloc)
    }

    /// Create a new empty [`Vec`] inside the builder's memory arena with at
    /// least the specified capacity.
    ///
    /// This [`Vec`] will be able to hold at least `capacity` elements without re-allocating.
    #[must_use]
    pub fn vec_with_capacity<T>(&self, capacity: usize) -> Vec<'a, T> {
        Vec::with_capacity_in(capacity, self.alloc)
    }

    pub fn vec_from_slice<T>(&self, slice: &[T]) -> Vec<'a, T>
    where
        T: Copy,
    {
        let mut v = Vec::with_capacity_in(slice.len(), self.alloc);
        v.copy_from_slice(slice);
        v
    }

    /// Create a new empty [`String`] inside the builder's memory arena.
    #[must_use]
    pub fn string(&self) -> String<'a> {
        String::new_in(self.alloc)
    }

    /// Construct a new `String<'bump>` from a string slice.
    #[must_use]
    pub fn string_from(&self, s: &str) -> String<'a> {
        String::from_str_in(s, self.alloc)
    }

    /// Creates a new empty [`String`] with a particular capacity.
    ///
    /// `String`s have an internal buffer to hold their data. The capacity is
    /// the length of that buffer, and can be queried with the [`capacity`]
    /// method. This method creates an empty `String`, but one with an initial
    /// buffer that can hold `capacity` bytes. This is useful when you may be
    /// appending a bunch of data to the `String`, reducing the number of
    /// reallocations it needs to do.
    #[must_use]
    pub fn string_with_capacity(&self, capacity: usize) -> String<'a> {
        String::with_capacity_in(capacity, self.alloc)
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
        self.table_mut().create_type(ty, flags, None, None, None)
    }

    pub fn create_literal_type<V>(
        &self,
        flags: TypeFlags,
        value: V,
        symbol: Option<SymbolId>,
        regular_type: Option<TypeId>,
    ) -> TypeId
    where
        V: Into<FreshLiteralType<'a>>,
    {
        // function createLiteralType(flags: TypeFlags, value: string | number | PseudoBigInt, symbol?: Symbol, regularType?: LiteralType) {
        //     const type = createTypeWithSymbol(flags, symbol!) as LiteralType;
        //     type.value = value;
        //     type.regularType = regularType || type;
        //     return type;
        // }
        let ty = Type::Literal(self.alloc(LiteralType::Fresh(value.into(), regular_type)));
        self.table_mut().create_type(
            ty, flags, symbol, /* alias_symbol */ None, /* alias_type_arguments */ None,
        )
    }

    /// Creates a [`UnionType`]
    pub fn create_union_type(
        &self,
        types: &[TypeId],
        // union_reduction: UnionReduction,
        object_flags: ObjectFlags,
        alias_symbol: Option<SymbolId>,
        alias_type_arguments: Option<&[TypeId]>,
        origin: Option<TypeId>,
    ) -> TypeId {
        assert!(types.len() > 1, "Union types must have at least two members");

        let ty = Type::Union(self.alloc(UnionType {
            types: self.vec_from_slice(types),
            object_flags,
            origin,
        }));

        self.table_mut().create_type(
            ty,
            TypeFlags::Union,
            None,
            alias_symbol,
            alias_type_arguments.map(|args| self.vec_from_slice(args)),
        )
    }
}

impl fmt::Debug for TypeBuilder<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypeBuilder").field("table", &self.table).finish()
    }
}
