//! Union and intersection types
use std::cell::Ref;

use oxc_allocator::Vec;
use oxc_semantic::SymbolId;
use oxc_syntax::types::{TypeFlags, TypeId};

use crate::ast::Type;

use super::Checker;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum UnionReduction {
    None = 0,
    #[default]
    Literal,
    Subtype,
}

impl UnionReduction {
    #[inline]
    pub const fn is_none(self) -> bool {
        matches!(self, Self::None)
    }

    #[inline]
    pub const fn is_literal(self) -> bool {
        matches!(self, Self::Literal)
    }

    #[inline]
    pub const fn is_subtype(self) -> bool {
        matches!(self, Self::Subtype)
    }
}

impl<'a> Checker<'a> {
    pub fn get_union_type(
        &self,
        types: &[TypeId],
        union_reduction: UnionReduction,
        alias_symbol: Option<SymbolId>,
        type_alias_arguments: Option<&[TypeId]>,
        origin: Option<TypeId>,
    ) -> TypeId {
        match types.len() {
            0 => self.intrinsics.never,
            1 => types[0],
            _ => self.get_union_type_worker(
                types,
                union_reduction,
                alias_symbol,
                type_alias_arguments,
                origin,
            ),
        }
    }

    fn get_union_type_worker(
        &self,
        types: &[TypeId],
        union_reduction: UnionReduction,
        alias_symbol: Option<SymbolId>,
        type_alias_arguments: Option<&[TypeId]>,
        origin: Option<TypeId>,
    ) -> TypeId {
        todo!()
    }

    /// Add the given types to the given type set. Order is preserved,
    /// duplicates are removed, and nested types of the given kind are flattened
    /// into the set.
    fn add_types_to_union(
        &self,
        type_set: &mut Vec<'a, TypeId>,
        includes: TypeFlags,
        types: &[TypeId],
    ) {
        /*
        function addTypesToUnion(typeSet: Type[], includes: TypeFlags, types: readonly Type[]): TypeFlags {
            let lastType: Type | undefined;
            for (const type of types) {
                // We skip the type if it is the same as the last type we processed. This simple test particularly
                // saves a lot of work for large lists of the same union type, such as when resolving `Record<A, B>[A]`,
                // where A and B are large union types.
                if (type !== lastType) {
                    includes = type.flags & TypeFlags.Union ?
                        addTypesToUnion(typeSet, includes | (isNamedUnionType(type) ? TypeFlags.Union : 0), (type as UnionType).types) :
                        addTypeToUnion(typeSet, includes, type);
                    lastType = type;
                }
            }
            return includes;
        }
         */
        let mut last_type: Option<TypeId> = None;
        for ty in types.iter().copied() {
            if last_type.is_some_and(|last_type| last_type == ty) {
                continue;
            }
            if self.get_flags(ty).is_union() {
                let union = Ref::map(self.get_type(ty), Type::into_union);
                let includes = if self.is_named_union_type(&union) {
                    TypeFlags::Union
                } else {
                    TypeFlags::empty()
                };
                self.add_types_to_union(&type_set)
            }
        }
    }
    // function isNamedUnionType(type: Type) {
    //     return !!(type.flags & TypeFlags.Union && (type.aliasSymbol || (type as UnionType).origin));
    // }
    fn is_named_union_type(&self, union: &UnionType<'a>) -> bool {
        todo!("is_named_union_type")
    }

    /*
    function addTypeToUnion(typeSet: Type[], includes: TypeFlags, type: Type) {
        const flags = type.flags;
        // We ignore 'never' types in unions
        if (!(flags & TypeFlags.Never)) {
            includes |= flags & TypeFlags.IncludesMask;
            if (flags & TypeFlags.Instantiable) includes |= TypeFlags.IncludesInstantiable;
            if (flags & TypeFlags.Intersection && getObjectFlags(type) & ObjectFlags.IsConstrainedTypeVariable) includes |= TypeFlags.IncludesConstrainedTypeVariable;
            if (type === wildcardType) includes |= TypeFlags.IncludesWildcard;
            if (isErrorType(type)) includes |= TypeFlags.IncludesError;
            if (!strictNullChecks && flags & TypeFlags.Nullable) {
                if (!(getObjectFlags(type) & ObjectFlags.ContainsWideningType)) includes |= TypeFlags.IncludesNonWideningType;
            }
            else {
                const len = typeSet.length;
                const index = len && type.id > typeSet[len - 1].id ? ~len : binarySearch(typeSet, type, getTypeId, compareValues);
                if (index < 0) {
                    typeSet.splice(~index, 0, type);
                }
            }
        }
        return includes;
    }

     */
    fn add_type_to_union(
        &self,
        type_set: &mut Vec<'a, TypeId>,
        includes: TypeFlags,
        ty: TypeId,
    ) -> TypeFlags {
        todo!("add_type_to_union");
    }
}
