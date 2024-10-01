//! Methods for inquiring about a type.

use oxc_syntax::types::{ObjectFlags, TypeId};

use crate::ast::Type;

use super::Checker;

impl<'a> Checker<'a> {
    /// ```typescript
    /// function isErrorType(type: Type) {
    ///     return type === errorType || !!(type.flags & TypeFlags.Any && type.aliasSymbol);
    /// }
    /// ```
    pub fn is_error_type(&self, type_id: TypeId) -> bool {
        type_id == self.intrinsics.error
            || (self.get_flags(type_id).is_any()
                && self.builder.table().get_alias_symbol(type_id).is_some())
    }

    /// ```typescript
    /// function getObjectFlags(type: Type): ObjectFlags {
    ///     return type.flags & TypeFlags.ObjectFlagsType ? (type as ObjectFlagsType).objectFlags : 0;
    /// }
    /// ```
    pub fn get_object_flags(&self, type_id: TypeId) -> ObjectFlags {
        let flags = self.get_type(type_id).get_object_flags();

        #[cfg(debug_assertions)]
        if !flags.is_empty() {
            let type_flags = self.get_flags(type_id);
            assert!(type_flags.is_type_with_object_flags(), "Type {type_id:?} should not have object flags.\n\tTypeFlags: {type_flags:?}\n\tObjectFlags: {flags:?}");
        }

        flags
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
