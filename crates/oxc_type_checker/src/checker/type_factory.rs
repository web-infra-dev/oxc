use oxc_syntax::types::{ObjectFlags, TypeFlags, TypeId};

use super::Checker;

impl<'a> Checker<'a> {
    /// This function is used to propagate certain flags when creating new object type references and union types.
    /// It is only necessary to do so if a constituent type might be the undefined type, the null type, the type
    /// of an object literal or a non-inferrable type. This is because there are operations in the type checker
    /// that care about the presence of such types at arbitrary depth in a containing type.
    ///
    /// ```typescript
    /// function getPropagatingFlagsOfTypes(types: readonly Type[], excludeKinds?: TypeFlags): ObjectFlags {
    ///     let result: ObjectFlags = 0;
    ///     for (const type of types) {
    ///         if (excludeKinds === undefined || !(type.flags & excludeKinds)) {
    ///             result |= getObjectFlags(type);
    ///         }
    ///     }
    ///     return result & ObjectFlags.PropagatingFlags;
    /// }
    /// ```
    pub fn get_propagating_flags_of_types(
        &self,
        types: &[TypeId],
        exclude_kinds: Option<TypeFlags>,
    ) -> ObjectFlags {
        let mut result = ObjectFlags::empty();

        for &type_id in types {
            if exclude_kinds
                .map_or(true, |exclude_kinds| !self.get_flags(type_id).intersects(exclude_kinds))
            {
                result |= self.get_object_flags(type_id);
            }
        }

        result & ObjectFlags::PropagatingFlags
    }
}
