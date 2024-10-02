//! Methods for inquiring about a type.

use oxc_semantic::{SymbolFlags, SymbolId};
use oxc_syntax::types::{ObjectFlags, TypeId};

use crate::ast::Type;

use super::Checker;

impl<'a> Checker<'a> {
    /// ```typescript
    /// function isErrorType(type: Type) {
    ///     return type === errorType || !!(type.flags & TypeFlags.Any && type.aliasSymbol);
    /// }
    /// ```
    pub(crate) fn is_error_type(&self, type_id: TypeId) -> bool {
        type_id == self.intrinsics.error
            || (self.get_flags(type_id).is_any()
                && self.builder.table().get_alias_symbol(type_id).is_some())
    }

    /// ```typescript
    /// function getObjectFlags(type: Type): ObjectFlags {
    ///     return type.flags & TypeFlags.ObjectFlagsType ? (type as ObjectFlagsType).objectFlags : 0;
    /// }
    /// ```
    pub(crate) fn get_object_flags(&self, type_id: TypeId) -> ObjectFlags {
        let flags = self.get_type(type_id).get_object_flags();

        #[cfg(debug_assertions)]
        if !flags.is_empty() {
            let type_flags = self.get_flags(type_id);
            assert!(type_flags.is_type_with_object_flags(), "Type {type_id:?} should not have object flags.\n\tTypeFlags: {type_flags:?}\n\tObjectFlags: {flags:?}");
        }

        flags
    }

    /// ```typescript
    /// function isConstEnumObjectType(type: Type): boolean {
    ///     return !!(getObjectFlags(type) & ObjectFlags.Anonymous) && !!type.symbol && isConstEnumSymbol(type.symbol);
    /// }
    /// ```
    pub(crate) fn is_const_enum_object_type(&self, type_id: TypeId) -> bool {
        self.get_object_flags(type_id).is_anonymous()
            && self
                .builder
                .table()
                .get_symbol(type_id)
                .is_some_and(|symbol_id| self.is_const_enum_symbol(symbol_id))
    }

    /// ```typescript
    /// function isConstEnumSymbol(symbol: Symbol): boolean {
    ///     return (symbol.flags & SymbolFlags.ConstEnum) !== 0;
    /// }
    /// ```
    #[inline]
    pub(crate) fn is_const_enum_symbol(&self, symbol_id: SymbolId) -> bool {
        self.semantic.symbols().get_flags(symbol_id).contains(SymbolFlags::ConstEnum)
    }
}

impl<'a> Type<'a> {
    /// NOTE: object flags may be moved into the type table. This method should
    /// remain private, and consumers should use [`Checker::get_object_flags`].
    fn get_object_flags(&self) -> ObjectFlags {
        match self {
            Self::Intrinsic(intrinsic) => intrinsic.object_flags,
            Self::Union(union) => union.object_flags,
            Self::Object(object) => object.object_flags,
            _ => ObjectFlags::empty(),
        }
    }
}
