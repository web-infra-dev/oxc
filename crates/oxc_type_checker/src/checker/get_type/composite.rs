//! Union and intersection types

use oxc_allocator::{String, Vec};
use oxc_semantic::SymbolId;
use oxc_syntax::types::{ObjectFlags, TypeFlags, TypeId};

use crate::ast::{Type, UnionType};

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
    /// [`Checker::get_union_type`], but with default values for optional/variadic arguments.
    #[inline]
    #[must_use]
    pub(crate) fn get_union_type_simple(&self, types: &[TypeId]) -> TypeId {
        self.get_union_type(types, UnionReduction::Literal, None, None, None)
    }

    /// From TypeScript:
    ///
    /// > We sort and deduplicate the constituent types based on object identity. If the subtypeReduction
    /// > flag is specified we also reduce the constituent type set to only include types that aren't subtypes
    /// > of other types. Subtype reduction is expensive for large union types and is possible only when union
    /// > types are known not to circularly reference themselves (as is the case with union types created by
    /// > expression constructs such as array literals and the || and ?: operators). Named types can
    /// > circularly reference themselves and therefore cannot be subtype reduced during their declaration.
    /// > For example, "type Item = string | (() => Item" is a named type that circularly references itself.
    ///
    /// ```typescript
    /// function getUnionType(types: readonly Type[], unionReduction: UnionReduction = UnionReduction.Literal, aliasSymbol?: Symbol, aliasTypeArguments?: readonly Type[], origin?: Type): Type {
    ///     if (types.length === 0) {
    ///         return neverType;
    ///     }
    ///     if (types.length === 1) {
    ///         return types[0];
    ///     }
    ///     // We optimize for the common case of unioning a union type with some other type (such as `undefined`).
    ///     if (types.length === 2 && !origin && (types[0].flags & TypeFlags.Union || types[1].flags & TypeFlags.Union)) {
    ///         const infix = unionReduction === UnionReduction.None ? "N" : unionReduction === UnionReduction.Subtype ? "S" : "L";
    ///         const index = types[0].id < types[1].id ? 0 : 1;
    ///         const id = types[index].id + infix + types[1 - index].id + getAliasId(aliasSymbol, aliasTypeArguments);
    ///         let type = unionOfUnionTypes.get(id);
    ///         if (!type) {
    ///             type = getUnionTypeWorker(types, unionReduction, aliasSymbol, aliasTypeArguments, /*origin*/ undefined);
    ///             unionOfUnionTypes.set(id, type);
    ///         }
    ///         return type;
    ///     }
    ///     return getUnionTypeWorker(types, unionReduction, aliasSymbol, aliasTypeArguments, origin);
    /// }
    /// ```
    #[must_use]
    pub(crate) fn get_union_type(
        &self,
        types: &[TypeId],
        union_reduction: UnionReduction,
        alias_symbol: Option<SymbolId>,
        // type arguments (generics) passed to one (maybe more?) of the types in
        // the union. Only present when one of those types is not pre-computed.
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

    /// ```typescript
    /// function getUnionTypeWorker(types: readonly Type[], unionReduction: UnionReduction, aliasSymbol: Symbol | undefined, aliasTypeArguments: readonly Type[] | undefined, origin: Type | undefined): Type {
    ///     let typeSet: Type[] | undefined = [];
    ///     const includes = addTypesToUnion(typeSet, 0 as TypeFlags, types);
    ///     if (unionReduction !== UnionReduction.None) {
    ///         if (includes & TypeFlags.AnyOrUnknown) {
    ///             return includes & TypeFlags.Any ?
    ///                 includes & TypeFlags.IncludesWildcard ? wildcardType :
    ///                     includes & TypeFlags.IncludesError ? errorType : anyType :
    ///                 unknownType;
    ///         }
    ///         if (includes & TypeFlags.Undefined) {
    ///             // If type set contains both undefinedType and missingType, remove missingType
    ///             if (typeSet.length >= 2 && typeSet[0] === undefinedType && typeSet[1] === missingType) {
    ///                 orderedRemoveItemAt(typeSet, 1);
    ///             }
    ///         }
    ///         if (includes & (TypeFlags.Enum | TypeFlags.Literal | TypeFlags.UniqueESSymbol | TypeFlags.TemplateLiteral | TypeFlags.StringMapping) || includes & TypeFlags.Void && includes & TypeFlags.Undefined) {
    ///             removeRedundantLiteralTypes(typeSet, includes, !!(unionReduction & UnionReduction.Subtype));
    ///         }
    ///         if (includes & TypeFlags.StringLiteral && includes & (TypeFlags.TemplateLiteral | TypeFlags.StringMapping)) {
    ///             removeStringLiteralsMatchedByTemplateLiterals(typeSet);
    ///         }
    ///         if (includes & TypeFlags.IncludesConstrainedTypeVariable) {
    ///             removeConstrainedTypeVariables(typeSet);
    ///         }
    ///         if (unionReduction === UnionReduction.Subtype) {
    ///             typeSet = removeSubtypes(typeSet, !!(includes & TypeFlags.Object));
    ///             if (!typeSet) {
    ///                 return errorType;
    ///             }
    ///         }
    ///         if (typeSet.length === 0) {
    ///             return includes & TypeFlags.Null ? includes & TypeFlags.IncludesNonWideningType ? nullType : nullWideningType :
    ///                 includes & TypeFlags.Undefined ? includes & TypeFlags.IncludesNonWideningType ? undefinedType : undefinedWideningType :
    ///                 neverType;
    ///         }
    ///     }
    ///     if (!origin && includes & TypeFlags.Union) {
    ///         const namedUnions: Type[] = [];
    ///         addNamedUnions(namedUnions, types);
    ///         const reducedTypes: Type[] = [];
    ///         for (const t of typeSet) {
    ///             if (!some(namedUnions, union => containsType((union as UnionType).types, t))) {
    ///                 reducedTypes.push(t);
    ///             }
    ///         }
    ///         if (!aliasSymbol && namedUnions.length === 1 && reducedTypes.length === 0) {
    ///             return namedUnions[0];
    ///         }
    ///         // We create a denormalized origin type only when the union was created from one or more named unions
    ///         // (unions with alias symbols or origins) and when there is no overlap between those named unions.
    ///         const namedTypesCount = reduceLeft(namedUnions, (sum, union) => sum + (union as UnionType).types.length, 0);
    ///         if (namedTypesCount + reducedTypes.length === typeSet.length) {
    ///             for (const t of namedUnions) {
    ///                 insertType(reducedTypes, t);
    ///             }
    ///             origin = createOriginUnionOrIntersectionType(TypeFlags.Union, reducedTypes);
    ///         }
    ///     }
    ///     const objectFlags = (includes & TypeFlags.NotPrimitiveUnion ? 0 : ObjectFlags.PrimitiveUnion) |
    ///         (includes & TypeFlags.Intersection ? ObjectFlags.ContainsIntersections : 0);
    ///     return getUnionTypeFromSortedList(typeSet, objectFlags, aliasSymbol, aliasTypeArguments, origin);
    /// }
    /// ```
    fn get_union_type_worker(
        &self,
        types: &[TypeId],
        union_reduction: UnionReduction,
        alias_symbol: Option<SymbolId>,
        type_alias_arguments: Option<&[TypeId]>,
        origin: Option<TypeId>,
    ) -> TypeId {
        assert!(types.len() > 1);
        let mut type_set = self.builder.vec();
        let includes = self.add_types_to_union(&mut type_set, TypeFlags::empty(), types);

        if !union_reduction.is_none() {
            // if (includes & TypeFlags.AnyOrUnknown) {
            //     return includes & TypeFlags.Any ?
            //         includes & TypeFlags.IncludesWildcard ? wildcardType :
            //             includes & TypeFlags.IncludesError ? errorType : anyType :
            //         unknownType;
            // }
            if includes.is_any_or_unknown() {
                return if includes.is_any() {
                    if includes.includes_wildcard() {
                        self.intrinsics.wildcard
                    } else if includes.includes_error() {
                        self.intrinsics.error
                    } else {
                        self.intrinsics.any
                    }
                } else {
                    self.intrinsics.unknown
                };
            }

            if includes.is_undefined() {
                // If type set contains both undefinedType and missingType, remove missingType
                if type_set.len() > 2
                    && type_set[0] == self.intrinsics.undefined
                    && type_set[1] == self.intrinsics.missing
                {
                    type_set.remove(1);
                }
            }

            if includes.intersects(
                TypeFlags::Enum
                    | TypeFlags::Literal
                    | TypeFlags::UniqueESSymbol
                    | TypeFlags::TemplateLiteral
                    | TypeFlags::StringMapping,
            ) || (includes.is_void() && includes.is_undefined())
            {
                // removeRedundantLiteralTypes(typeSet, includes, // !!(unionReduction & UnionReduction.Subtype));

                todo!("removeRedundantLiteralTypes(typeSet, includes, // !!(unionReduction & UnionReduction.Subtype))")
            }

            if includes.intersects(TypeFlags::StringLiteral)
                && includes.intersects(TypeFlags::TemplateLiteral | TypeFlags::StringMapping)
            {
                // removeStringLiteralsMatchedByTemplateLiterals(typeSet);
                todo!("removeStringLiteralsMatchedByTemplateLiterals(typeSet)")
            }

            if includes.includes_constrained_type_variable() {
                // removeConstrainedTypeVariables(typeSet);
                todo!("removeConstrainedTypeVariables(typeSet)")
            }

            if union_reduction.is_subtype() {
                // typeSet = removeSubtypes(typeSet, !!(includes & TypeFlags.Object));
                // if (!typeSet) {
                //     return errorType;
                // }
                todo!("typeSet = removeSubtypes(typeSet, !!(includes & TypeFlags.Object))")
            }

            if type_set.is_empty() {
                // return includes & TypeFlags.Null ? includes & TypeFlags.IncludesNonWideningType ? nullType : nullWideningType :
                //     includes & TypeFlags.Undefined ? includes & TypeFlags.IncludesNonWideningType ? undefinedType : undefinedWideningType :
                //     neverType;
                return if includes.is_null() {
                    if includes.includes_non_widening_type() {
                        self.intrinsics.null
                    } else {
                        self.intrinsics.null_widening
                    }
                } else if includes.is_undefined() {
                    if includes.includes_non_widening_type() {
                        self.intrinsics.undefined
                    } else {
                        self.intrinsics.undefined_widening
                    }
                } else {
                    self.intrinsics.never
                };
            }
        }

        if origin.is_none() && includes.is_union() {
            // const namedUnions: Type[] = [];
            // addNamedUnions(namedUnions, types);
            // const reducedTypes: Type[] = [];
            // for (const t of typeSet) {
            //     if (!some(namedUnions, union => containsType((union as UnionType).types, t))) {
            //         reducedTypes.push(t);
            //     }
            // }
            // if (!aliasSymbol && namedUnions.length === 1 && reducedTypes.length === 0) {
            //     return namedUnions[0];
            // }
            // // We create a denormalized origin type only when the union was created from one or more named unions
            // // (unions with alias symbols or origins) and when there is no overlap between those named unions.
            // const namedTypesCount = reduceLeft(namedUnions, (sum, union) => sum + (union as UnionType).types.length, 0);
            // if (namedTypesCount + reducedTypes.length === typeSet.length) {
            //     for (const t of namedUnions) {
            //         insertType(reducedTypes, t);
            //     }
            //     origin = createOriginUnionOrIntersectionType(TypeFlags.Union, reducedTypes);
            // }
            todo!("origin.is_none() && includes.is_union()")
        }

        // const objectFlags = (includes & TypeFlags.NotPrimitiveUnion ? 0 : ObjectFlags.PrimitiveUnion) |
        //     (includes & TypeFlags.Intersection ?
        //     ObjectFlags.ContainsIntersections : 0);
        let object_flags = ObjectFlags::empty()
            .with_primitive_union(!includes.is_non_primitive_union())
            .with_contains_intersections(includes.is_intersection());

        self.get_union_type_from_sorted_list(
            &type_set,
            object_flags,
            alias_symbol,
            type_alias_arguments,
            origin,
        )
    }

    /// Add the given types to the given type set. Order is preserved,
    /// duplicates are removed, and nested types of the given kind are flattened
    /// into the set.
    ///
    /// ```typescript
    /// function addTypesToUnion(typeSet: Type[], includes: TypeFlags, types: readonly Type[]): TypeFlags {
    ///     let lastType: Type | undefined;
    ///     for (const type of types) {
    ///         // We skip the type if it is the same as the last type we processed. This simple test particularly
    ///         // saves a lot of work for large lists of the same union type, such as when resolving `Record<A, B>[A]`,
    ///         // where A and B are large union types.
    ///         if (type !== lastType) {
    ///             includes = type.flags & TypeFlags.Union ?
    ///                 addTypesToUnion(typeSet, includes | (isNamedUnionType(type) ? TypeFlags.Union : 0), (type as UnionType).types) :
    ///                 addTypeToUnion(typeSet, includes, type);
    ///             lastType = type;
    ///         }
    ///     }
    ///     return includes;
    /// }
    /// ```
    fn add_types_to_union(
        &self,
        type_set: &mut Vec<'a, TypeId>,
        includes: TypeFlags,
        types: &[TypeId],
    ) -> TypeFlags {
        let mut last_type: Option<TypeId> = None;
        let mut includes = includes;

        for ty_id in types.iter().copied() {
            if last_type.is_some_and(|last_type| last_type == ty_id) {
                continue;
            }

            let ty = self.get_type(ty_id);
            match &*ty {
                Type::Union(union) => {
                    let new_includes = if self.is_named_union_type(union, ty_id) {
                        includes | TypeFlags::Union
                    } else {
                        includes
                    };
                    includes =
                        self.add_types_to_union(type_set, new_includes, union.types.as_slice());
                }
                _ => {
                    includes = self.add_type_to_union(type_set, includes, ty_id);
                }
            };

            last_type.replace(ty_id);
        }

        includes
    }

    ///```typescript
    /// function isNamedUnionType(type: Type) {
    ///     return !!(type.flags & TypeFlags.Union && (type.aliasSymbol || (type as UnionType).origin));
    /// }
    /// ```
    fn is_named_union_type(&self, union: &UnionType<'a>, id: TypeId) -> bool {
        union.origin.is_some() || self.builder.table().get_alias_symbol(id).is_some()
    }

    /// ```typescript
    /// function addTypeToUnion(typeSet: Type[], includes: TypeFlags, type: Type) {
    ///     const flags = type.flags;
    ///     // We ignore 'never' types in unions
    ///     if (!(flags & TypeFlags.Never)) {
    ///         includes |= flags & TypeFlags.IncludesMask;
    ///         if (flags & TypeFlags.Instantiable) includes |= TypeFlags.IncludesInstantiable;
    ///         if (flags & TypeFlags.Intersection && getObjectFlags(type) & ObjectFlags.IsConstrainedTypeVariable) includes |= TypeFlags.IncludesConstrainedTypeVariable;
    ///         if (type === wildcardType) includes |= TypeFlags.IncludesWildcard;
    ///         if (isErrorType(type)) includes |= TypeFlags.IncludesError;
    ///         if (!strictNullChecks && flags & TypeFlags.Nullable) {
    ///             if (!(getObjectFlags(type) & ObjectFlags.ContainsWideningType)) includes |= TypeFlags.IncludesNonWideningType;
    ///         }
    ///         else {
    ///             const len = typeSet.length;
    ///             const index = len && type.id > typeSet[len - 1].id ? ~len : binarySearch(typeSet, type, getTypeId, compareValues);
    ///             if (index < 0) {
    ///                 typeSet.splice(~index, 0, type);
    ///             }
    ///         }
    ///     }
    ///     return includes;
    /// }
    ///```
    fn add_type_to_union(
        &self,
        type_set: &mut Vec<'a, TypeId>,
        includes: TypeFlags,
        ty: TypeId,
    ) -> TypeFlags {
        let flags = self.get_flags(ty);
        // We ignore 'never' types in unions
        if flags.is_never() {
            return includes;
        }

        let mut includes = flags.mask_for_includes();

        // if (flags & TypeFlags.Instantiable) includes |= TypeFlags.IncludesInstantiable;
        if flags.is_instantiable() {
            includes |= TypeFlags::IncludesInstantiable;
        }

        // if (flags & TypeFlags.Intersection && getObjectFlags(type) & ObjectFlags.IsConstrainedTypeVariable) includes |= TypeFlags.IncludesConstrainedTypeVariable;
        if flags.is_intersection() && self.get_object_flags(ty).is_constrained_type_variable() {
            includes |= TypeFlags::IncludesConstrainedTypeVariable;
        }

        // if (type === wildcardType) includes |= TypeFlags.IncludesWildcard;
        if ty == self.intrinsics.wildcard {
            includes |= TypeFlags::IncludesWildcard;
        }

        // if (isErrorType(type)) includes |= TypeFlags.IncludesError;
        if self.is_error_type(ty) {
            includes |= TypeFlags::IncludesError;
        }

        //  if (!strictNullChecks && flags & TypeFlags.Nullable) {
        //      if (!(getObjectFlags(type) & ObjectFlags.ContainsWideningType)) includes |= TypeFlags.IncludesNonWideningType;
        //  }
        if !self.settings.strict_null_checks && flags.is_nullable() {
            todo!("strictNullChecks: false is not supported rn");
        } else {
            // const len = typeSet.length;
            // const index = len && type.id > typeSet[len - 1].id ? ~len : binarySearch(typeSet, type, getTypeId, compareValues);
            // if (index < 0) {
            //     typeSet.splice(~index, 0, type);
            // }
            type_set.push(ty);
            type_set.sort_unstable();
        }

        includes
    }

    /// This function assumes the constituent type list is sorted and deduplicated.
    /// ```typescript
    /// function getUnionTypeFromSortedList(types: Type[], precomputedObjectFlags: ObjectFlags, aliasSymbol?: Symbol, aliasTypeArguments?: readonly Type[], origin?: Type): Type {
    ///     if (types.length === 0) {
    ///         return neverType;
    ///     }
    ///     if (types.length === 1) {
    ///         return types[0];
    ///     }
    ///     const typeKey = !origin ? getTypeListId(types) :
    ///         origin.flags & TypeFlags.Union ? `|${getTypeListId((origin as UnionType).types)}` :
    ///         origin.flags & TypeFlags.Intersection ? `&${getTypeListId((origin as IntersectionType).types)}` :
    ///         `#${(origin as IndexType).type.id}|${getTypeListId(types)}`; // origin type id alone is insufficient, as `keyof x` may resolve to multiple WIP values while `x` is still resolving
    ///     const id = typeKey + getAliasId(aliasSymbol, aliasTypeArguments);
    ///     let type = unionTypes.get(id);
    ///     if (!type) {
    ///         type = createType(TypeFlags.Union) as UnionType;
    ///         type.objectFlags = precomputedObjectFlags | getPropagatingFlagsOfTypes(types, /*excludeKinds*/ TypeFlags.Nullable);
    ///         type.types = types;
    ///         type.origin = origin;
    ///         type.aliasSymbol = aliasSymbol;
    ///         type.aliasTypeArguments = aliasTypeArguments;
    ///         if (types.length === 2 && types[0].flags & TypeFlags.BooleanLiteral && types[1].flags & TypeFlags.BooleanLiteral) {
    ///             type.flags |= TypeFlags.Boolean;
    ///             (type as UnionType & IntrinsicType).intrinsicName = "boolean";
    ///         }
    ///         unionTypes.set(id, type);
    ///     }
    ///     return type;
    /// }
    /// ```
    fn get_union_type_from_sorted_list(
        &self,
        types: &[TypeId],
        precomputed_object_flags: ObjectFlags,
        alias_symbol: Option<SymbolId>,
        alias_symbol_arguments: Option<&[TypeId]>,
        origin: Option<TypeId>,
    ) -> TypeId {
        match types.len() {
            0 => self.intrinsics.never,
            1 => types[0],
            _ => {
                // TODO: get/set union type cache entry
                // let type_key = if let Some(origin) = origin {
                //     let origin_ty = self.get_type(origin);
                //     match &*origin_ty {
                //         Type::Union(union) => {
                //             self.builder.string_from("|")
                //                 + &self.get_type_list_id(Some(union.types.as_slice()))
                //         }
                //         // TODO: intersection types
                //         // Type::Intersection(intersection) => {
                //         //     // self.builformat!("&{}",
                //         //     // self.get_type_list_id(intersection.types.as_slice()))
                //         //     self.builder.string_from("&")
                //         //         + &self.get_type_list_id(Some(intersection.types.as_slice()))
                //         // }
                //         _ => {
                //             // let origin = self.get_type(origin);
                //             // self.builder.string_from("#") +
                //             // &origin.to_string()
                //             self.builder.string_from(&format!(
                //                 "#{origin}|{}",
                //                 self.get_type_list_id(Some(types))
                //             ))
                //         }
                //     }
                // } else {
                //     self.get_type_list_id(Some(types))
                // };
                let object_flags = precomputed_object_flags
                    .union(self.get_propagating_flags_of_types(types, Some(TypeFlags::Nullable)));

                self.builder.create_union_type(
                    types,
                    object_flags,
                    alias_symbol,
                    alias_symbol_arguments,
                    origin,
                )
            }
        }
    }

    // NOTE: This is getting moved to TypeCache

    /// ```typescript
    /// function getTypeListId(types: readonly Type[] | undefined) {
    ///     let result = "";
    ///     if (types) {
    ///         const length = types.length;
    ///         let i = 0;
    ///         while (i < length) {
    ///             const startId = types[i].id;
    ///             let count = 1;
    ///             while (i + count < length && types[i + count].id === startId + count) {
    ///                 count++;
    ///             }
    ///             if (result.length) {
    ///                 result += ",";
    ///             }
    ///             result += startId;
    ///             if (count > 1) {
    ///                 result += ":" + count;
    ///             }
    ///             i += count;
    ///         }
    ///     }
    ///     return result;
    /// }
    /// ```
    fn get_type_list_id(&self, types: Option<&[TypeId]>) -> String<'a> {
        let Some(types) = types else {
            return self.builder.string();
        };
        let mut result = self.builder.string_with_capacity(types.len() * 2);
        let mut i = 0;
        // for (i, start_id) in types.into_iter().enumerate() {
        while i < types.len() {
            let start_id = types[i];
            let mut count = 1;
            while i + count < types.len() && types[i + count] == usize::from(start_id) + count {
                count += 1;
            }
            if !result.is_empty() {
                result.push(',');
            }

            result = result + &start_id.to_string();
            if count > 1 {
                result.push(':');
                result += &count.to_string();
            }
            i += count;
        }

        result
    }

    /// ```typescript
    /// function getAliasId(aliasSymbol: Symbol | undefined, aliasTypeArguments: readonly Type[] | undefined) {
    ///     return aliasSymbol ? `@${getSymbolId(aliasSymbol)}` + (aliasTypeArguments ? `:${getTypeListId(aliasTypeArguments)}` : "") : "";
    /// }
    /// ```
    fn get_alias_id(
        &self,
        alias_symbol: Option<SymbolId>,
        alias_type_arguments: Option<&[TypeId]>,
    ) -> String<'a> {
        let Some(alias_symbol) = alias_symbol else {
            return self.builder.string();
        };

        let mut result = self.builder.string_from("@") + &alias_symbol.to_string();
        if let Some(alias_type_arguments) = alias_type_arguments {
            result.push(':');
            result += &self.get_type_list_id(Some(alias_type_arguments));
        }
        result
    }
}
