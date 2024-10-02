use oxc_span::Atom;
use oxc_syntax::types::{TypeFlags, TypeId};

use crate::{
    ast::{LiteralType, Number, PseudoBigInt, Type},
    Checker,
};

impl<'a> Checker<'a> {
    /// **NOTE:** not fully implemented
    ///
    /// ```typescript
    /// function getFreshTypeOfLiteralType(type: Type): Type {
    ///     if (type.flags & TypeFlags.Freshable) {
    ///         if (!(type as FreshableType).freshType) {
    ///             const freshType = createLiteralType(type.flags, (type as LiteralType).value, (type as LiteralType).symbol, type as LiteralType);
    ///             freshType.freshType = freshType;
    ///             (type as FreshableType).freshType = freshType;
    ///         }
    ///         return (type as FreshableType).freshType;
    ///     }
    ///     return type;
    /// }
    /// ```
    pub(crate) fn get_fresh_type_of_literal_type(&self, type_id: TypeId) -> TypeId {
        if !self.get_flags(type_id).is_freshable() {
            return type_id;
        }

        let ty = self.get_type(type_id);
        match &*ty {
            Type::Literal(lit) => match lit.as_ref() {
                LiteralType::Fresh(..) => type_id,
                LiteralType::Regular(_, fresh_type_id) => *fresh_type_id,
            },
            _ => type_id,
        }
    }

    // function getRegularTypeOfLiteralType(type: Type): Type {
    //     return type.flags & TypeFlags.Freshable ? (type as FreshableType).regularType :
    //         type.flags & TypeFlags.Union ? ((type as UnionType).regularType || ((type as UnionType).regularType = mapType(type, getRegularTypeOfLiteralType) as UnionType)) :
    //         type;
    // }

    // function isFreshLiteralType(type: Type) {
    //     return !!(type.flags & TypeFlags.Freshable) && (type as LiteralType).freshType === type;
    // }

    /// ```typescript
    /// function getStringLiteralType(value: string): StringLiteralType {
    ///     let type;
    ///     return stringLiteralTypes.get(value) ||
    ///         (stringLiteralTypes.set(value, type = createLiteralType(TypeFlags.StringLiteral, value) as StringLiteralType), type);
    /// }
    /// ```
    pub(crate) fn get_string_literal_type(&self, value: &Atom<'a>) -> TypeId {
        self.cache.get_string(value).unwrap_or_else(|| {
            let type_id = self.builder.create_literal_type(
                TypeFlags::StringLiteral,
                value.clone(),
                None,
                /* todo: set? */ None,
            );
            self.cache.set_string(value.clone(), type_id);
            type_id
        })
    }

    /// ```typescript
    /// function getNumberLiteralType(value: number): NumberLiteralType {
    ///     let type;
    ///     return numberLiteralTypes.get(value) ||
    ///         (numberLiteralTypes.set(value, type = createLiteralType(TypeFlags.NumberLiteral, value) as NumberLiteralType), type);
    /// }
    /// ```
    pub(crate) fn get_number_literal_type<N: Into<Number>>(&self, value: N) -> TypeId {
        let num = value.into();
        self.cache.get_number(&num).unwrap_or_else(|| {
            let ty = self.builder.create_literal_type(
                TypeFlags::NumberLiteral,
                num,
                None,
                /* should this get set? */ None,
            );
            self.cache.add_number(num, ty);
            ty
        })
    }

    /// ```typescript
    /// function getBigIntLiteralType(value: PseudoBigInt): BigIntLiteralType {
    ///     let type;
    ///     const key = pseudoBigIntToString(value);
    ///     return bigIntLiteralTypes.get(key) ||
    ///         (bigIntLiteralTypes.set(key, type = createLiteralType(TypeFlags.BigIntLiteral, value) as BigIntLiteralType), type);
    /// }
    /// ```
    pub(crate) fn get_big_int_literal_type(&self, value: &PseudoBigInt<'a>) -> TypeId {
        self.cache.get_big_int(&value.raw).unwrap_or_else(|| {
            let type_id = self.builder.create_literal_type(
                TypeFlags::BigIntLiteral,
                value.clone(),
                None,
                /* todo: set? */ None,
            );
            self.cache.set_big_int(value.raw.clone(), type_id);
            type_id
        })
    }
}
