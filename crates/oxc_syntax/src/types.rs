use std::fmt;

use bitflags::bitflags;
use nonmax::NonMaxU32;
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer};

use oxc_index::Idx;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TypeId(NonMaxU32);

impl Idx for TypeId {
    #[allow(clippy::cast_possible_truncation)]
    fn from_usize(idx: usize) -> Self {
        assert!(idx < u32::MAX as usize);
        // SAFETY: We just checked `idx` is valid for `NonMaxU32`
        Self(unsafe { NonMaxU32::new_unchecked(idx as u32) })
    }

    fn index(self) -> usize {
        self.0.get() as usize
    }
}

#[cfg(feature = "serialize")]
impl Serialize for TypeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.0.get())
    }
}

impl fmt::Display for TypeId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.get().fmt(f)
    }
}

impl PartialEq<usize> for TypeId {
    #[inline]
    fn eq(&self, other: &usize) -> bool {
        u32::try_from(*other).map_or(false, |other| self.0.get() == other)
    }
}

impl From<TypeId> for usize {
    #[inline]
    fn from(id: TypeId) -> Self {
        id.0.get() as usize
    }
}

pub trait GetTypeId {
    fn type_id(&self) -> TypeId;
}

bitflags! {

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct TypeFlags: u32 {
        const Any             = 1 << 0;
        const Unknown         = 1 << 1;
        const String          = 1 << 2;
        const Number          = 1 << 3;
        const Boolean         = 1 << 4;
        /// Numeric computed enum member value
        const Enum            = 1 << 5;
        const BigInt          = 1 << 6;
        const StringLiteral   = 1 << 7;
        const NumberLiteral   = 1 << 8;
        const BooleanLiteral  = 1 << 9;
        /// Always combined with StringLiteral, NumberLiteral, or Union
        const EnumLiteral     = 1 << 10;
        const BigIntLiteral   = 1 << 11;
        /// Type of symbol primitive introduced in ES6
        const ESSymbol        = 1 << 12;
        /// unique symbol
        const UniqueESSymbol  = 1 << 13;
        const Void            = 1 << 14;
        const Undefined       = 1 << 15;
        const Null            = 1 << 16;
        /// Never type
        const Never           = 1 << 17;
        /// Type parameter
        const TypeParameter   = 1 << 18;
        /// Object type
        const Object          = 1 << 19;
        /// Union (`T | U``)
        const Union           = 1 << 20;
        /// Intersection (`T & U`)
        const Intersection    = 1 << 21;
        /// `keyof T``
        const Index           = 1 << 22;
        /// `T[K]`
        const IndexedAccess   = 1 << 23;
        /// `T extends U ? X : Y``
        const Conditional     = 1 << 24;
        /// Type parameter substitution
        const Substitution    = 1 << 25;
        /// intrinsic object type
        const NonPrimitive    = 1 << 26;
        /// Template literal type
        const TemplateLiteral = 1 << 27;
        /// Uppercase/Lowercase type
        const StringMapping   = 1 << 28;
        /// @internal
        #[doc(hidden)]
        const Reserved1       = 1 << 29;
        /// @internal
        #[doc(hidden)]
        const Reserved2       = 1 << 30;

        const AnyOrUnknown = Self::Any.bits() | Self::Unknown.bits();
        const Nullable = Self::Undefined.bits() | Self::Null.bits();
        const Literal = Self::StringLiteral.bits() | Self::NumberLiteral.bits() | Self::BigIntLiteral.bits() | Self::BooleanLiteral.bits();
        const Unit = Self::Enum.bits() | Self::Literal.bits() | Self::UniqueESSymbol.bits() | Self::Nullable.bits();
        const Freshable = Self::Enum.bits() | Self::Literal.bits();
        const StringOrNumberLiteral = Self::StringLiteral.bits() | Self::NumberLiteral.bits();
        const StringOrNumberLiteralOrUnique = Self::StringLiteral.bits() | Self::NumberLiteral.bits() | Self::UniqueESSymbol.bits();
        const DefinitelyFalsy = Self::StringLiteral.bits() | Self::NumberLiteral.bits() | Self::BigIntLiteral.bits() | Self::BooleanLiteral.bits() | Self::Void.bits() | Self::Undefined.bits() | Self::Null.bits();
        const PossiblyFalsy = Self::DefinitelyFalsy.bits() | Self::String.bits() | Self::Number.bits() | Self::BigInt.bits() | Self::Boolean.bits();
        const Intrinsic = Self::Any.bits() | Self::Unknown.bits() | Self::String.bits() | Self::Number.bits() | Self::BigInt.bits() | Self::Boolean.bits() | Self::BooleanLiteral.bits() | Self::ESSymbol.bits() | Self::Void.bits() | Self::Undefined.bits() | Self::Null.bits() | Self::Never.bits() | Self::NonPrimitive.bits();
        const StringLike = Self::String.bits() | Self::StringLiteral.bits() | Self::TemplateLiteral.bits() | Self::StringMapping.bits();
        const NumberLike = Self::Number.bits() | Self::NumberLiteral.bits() | Self::Enum.bits();
        const BigIntLike = Self::BigInt.bits() | Self::BigIntLiteral.bits();
        const BooleanLike = Self::Boolean.bits() | Self::BooleanLiteral.bits();
        const EnumLike = Self::Enum.bits() | Self::EnumLiteral.bits();
        const ESSymbolLike = Self::ESSymbol.bits() | Self::UniqueESSymbol.bits();
        const VoidLike = Self::Void.bits() | Self::Undefined.bits();
        const Primitive = Self::StringLike.bits() | Self::NumberLike.bits() | Self::BigIntLike.bits() | Self::BooleanLike.bits() | Self::EnumLike.bits() | Self::ESSymbolLike.bits() | Self::VoidLike.bits() | Self::Null.bits();
        const DefinitelyNonNullable = Self::StringLike.bits() | Self::NumberLike.bits() | Self::BigIntLike.bits() | Self::BooleanLike.bits() | Self::EnumLike.bits() | Self::ESSymbolLike.bits() | Self::Object.bits() | Self::NonPrimitive.bits();
        const DisjointDomains = Self::NonPrimitive.bits() | Self::StringLike.bits() | Self::NumberLike.bits() | Self::BigIntLike.bits() | Self::BooleanLike.bits() | Self::ESSymbolLike.bits() | Self::VoidLike.bits() | Self::Null.bits();
        const UnionOrIntersection = Self::Union.bits() | Self::Intersection.bits();
        const StructuredType = Self::Object.bits() | Self::Union.bits() | Self::Intersection.bits();
        const TypeVariable = Self::TypeParameter.bits() | Self::IndexedAccess.bits();
        const InstantiableNonPrimitive = Self::TypeVariable.bits() | Self::Conditional.bits() | Self::Substitution.bits();
        const InstantiablePrimitive = Self::Index.bits() | Self::TemplateLiteral.bits() | Self::StringMapping.bits();
        const Instantiable = Self::InstantiableNonPrimitive.bits() | Self::InstantiablePrimitive.bits();
        const StructuredOrInstantiable = Self::StructuredType.bits() | Self::Instantiable.bits();
        const ObjectFlagsType = Self::Any.bits() | Self::Nullable.bits() | Self::Never.bits() | Self::Object.bits() | Self::Union.bits() | Self::Intersection.bits();
        const Simplifiable = Self::IndexedAccess.bits() | Self::Conditional.bits();
        const Singleton = Self::Any.bits() | Self::Unknown.bits() | Self::String.bits() | Self::Number.bits() | Self::Boolean.bits() | Self::BigInt.bits() | Self::ESSymbol.bits() | Self::Void.bits() | Self::Undefined.bits() | Self::Null.bits() | Self::Never.bits() | Self::NonPrimitive.bits();
        const Narrowable = Self::Any.bits() | Self::Unknown.bits() | Self::StructuredOrInstantiable.bits() | Self::StringLike.bits() | Self::NumberLike.bits() | Self::BigIntLike.bits() | Self::BooleanLike.bits() | Self::ESSymbol.bits() | Self::UniqueESSymbol.bits() | Self::NonPrimitive.bits();
        const IncludesMask = Self::Any.bits() | Self::Unknown.bits() | Self::Primitive.bits() | Self::Never.bits() | Self::Object.bits() | Self::Union.bits() | Self::Intersection.bits() | Self::NonPrimitive.bits() | Self::TemplateLiteral.bits() | Self::StringMapping.bits();
        const IncludesMissingType = Self::TypeParameter.bits();
        const IncludesNonWideningType = Self::Index.bits();
        const IncludesWildcard = Self::IndexedAccess.bits();
        const IncludesEmptyObject = Self::Conditional.bits();
        const IncludesInstantiable = Self::Substitution.bits();
        const IncludesConstrainedTypeVariable = Self::Reserved1.bits();
        const IncludesError = Self::Reserved2.bits();
        const NonPrimitiveUnion = Self::Any.bits() | Self::Unknown.bits() | Self::Void.bits() | Self::Never.bits() | Self::Object.bits() | Self::Intersection.bits() | Self::IncludesInstantiable.bits();
    }
}

bitflags! {
    /// Object flags further qualify [`TypeFlags`] for object types.
    ///
    /// From TypeScript
    /// > Types included in TypeFlags.ObjectFlagsType have an objectFlags property. Some ObjectFlags
    /// > are specific to certain types and reuse the same bit position. Those ObjectFlags require a check
    /// > for a certain TypeFlags value to determine their meaning.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct ObjectFlags: u32 {
        const None = 0;
        /// Class
        const Class = 1 << 0;
        /// Interface
        const Interface = 1 << 1;
        /// Generic type reference
        const Reference = 1 << 2;
        /// Synthesized generic tuple type
        const Tuple = 1 << 3;
        /// Anonymous
        const Anonymous = 1 << 4;
        /// Mapped
        const Mapped = 1 << 5;
        /// Instantiated anonymous or mapped type
        const Instantiated = 1 << 6;
        /// Originates in an object literal
        const ObjectLiteral = 1 << 7;
        /// Evolving array type
        const EvolvingArray = 1 << 8;
        /// Object literal pattern with computed properties
        const ObjectLiteralPatternWithComputedProperties = 1 << 9;
        /// Object contains a property from a reverse-mapped type
        const ReverseMapped = 1 << 10;
        /// Jsx attributes type
        const JsxAttributes = 1 << 11;
        /// Object type declared in JS - disables errors on read/write of nonexisting members
        const JSLiteral = 1 << 12;
        /// Fresh object literal
        const FreshLiteral = 1 << 13;
        /// Originates in an array literal
        const ArrayLiteral = 1 << 14;
        /// Union of only primitive types
        const PrimitiveUnion = 1 << 15;
        /// Type is or contains undefined or null widening type
        const ContainsWideningType = 1 << 16;
        /// Type is or contains object literal type
        const ContainsObjectOrArrayLiteral = 1 << 17;
        /// Type is or contains anyFunctionType or silentNeverType
        const NonInferrableType = 1 << 18;
        /// CouldContainTypeVariables flag has been computed
        const CouldContainTypeVariablesComputed = 1 << 19;
        /// Type could contain a type variable
        const CouldContainTypeVariables = 1 << 20;

        const ClassOrInterface = Self::Class.bits() | Self::Interface.bits();
        /// RequiresWidening = ContainsWideningType | ContainsObjectOrArrayLiteral
        const RequiresWidening = Self::ContainsWideningType.bits() | Self::ContainsObjectOrArrayLiteral.bits();
        /// PropagatingFlags = ContainsWideningType | ContainsObjectOrArrayLiteral | NonInferrableType
        const PropagatingFlags = Self::ContainsWideningType.bits() | Self::ContainsObjectOrArrayLiteral.bits() | Self::NonInferrableType.bits();
        /// InstantiatedMapped = Mapped | Instantiated
        const InstantiatedMapped = Self::Mapped.bits() | Self::Instantiated.bits();
        /// ObjectTypeKindMask = ClassOrInterface | Reference | Tuple | Anonymous | Mapped | ReverseMapped | EvolvingArray
        const ObjectTypeKindMask = Self::ClassOrInterface.bits() | Self::Reference.bits() | Self::Tuple.bits() | Self::Anonymous.bits() | Self::Mapped.bits() | Self::ReverseMapped.bits() | Self::EvolvingArray.bits();

        // Flags that require TypeFlags.Object

        /// Object literal contains spread operation
        const ContainsSpread = 1 << 21;
        /// Originates in object rest declaration
        const ObjectRestType = 1 << 22;
        /// Originates in instantiation expression
        const InstantiationExpressionType = 1 << 23;
        /// A single signature type extracted from a potentially broader type
        const SingleSignatureType = 1 << 27;
        /// Type is a clone of a class instance type
        const IsClassInstanceClone = 1 << 24;
        /// has had `getSingleBaseForNonAugmentingSubtype` invoked on it already
        const IdenticalBaseTypeCalculated = 1 << 25;
        /// has a defined cachedEquivalentBaseType member
        const IdenticalBaseTypeExists = 1 << 26;

        // Flags that require TypeFlags.UnionOrIntersection or
        // TypeFlags.Substitution

        /// IsGenericObjectType flag has been computed
        const IsGenericTypeComputed = 1 << 21;
        /// Union or intersection contains generic object type
        const IsGenericObjectType = 1 << 22;
        /// Union or intersection contains generic index type
        const IsGenericIndexType = 1 << 23;
        /// IsGenericType = IsGenericObjectType | IsGenericIndexType
        const IsGenericType = Self::IsGenericObjectType.bits() | Self::IsGenericIndexType.bits();

        // Flags that require TypeFlags.Union

        /// Union contains intersections
        const ContainsIntersections = 1 << 24;
        /// IsUnknownLikeUnion flag has been computed
        const IsUnknownLikeUnionComputed = 1 << 25;
        /// Union of null, undefined, and empty object type
        const IsUnknownLikeUnion = 1 << 26;

        // Flags that require TypeFlags.Intersection

        /// IsNeverIntersectionComputed = 1 << 24
        const IsNeverIntersectionComputed = 1 << 24;
        /// Intersection reduces to never
        const IsNeverIntersection = 1 << 25;
        /// `T & C`, where T's constraint and C are primitives, object, or {}
        const IsConstrainedTypeVariable = 1 << 26;
    }
}

impl TypeFlags {
    // =========================================================================
    // ========================== SIMPLE FLAG CHECKS ===========================
    // =========================================================================

    #[inline]
    pub const fn is_any(self) -> bool {
        self.contains(Self::Any)
    }

    #[inline]
    pub const fn is_unknown(self) -> bool {
        self.contains(Self::Unknown)
    }

    #[inline]
    pub const fn is_never(self) -> bool {
        self.contains(Self::Never)
    }

    #[inline]
    pub const fn is_void(self) -> bool {
        self.contains(Self::Void)
    }

    #[inline]
    pub const fn is_undefined(self) -> bool {
        self.contains(Self::Undefined)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.contains(Self::Null)
    }

    #[inline]
    pub const fn is_nullable(self) -> bool {
        self.contains(Self::Nullable)
    }

    #[inline]
    pub const fn is_union(self) -> bool {
        self.contains(Self::Union)
    }

    #[inline]
    pub const fn is_intersection(self) -> bool {
        self.contains(Self::Intersection)
    }

    // =========================================================================
    // ========================= COMPOUND FLAG CHECKS ==========================
    // =========================================================================

    #[inline]
    pub const fn is_any_or_unknown(self) -> bool {
        self.intersects(Self::AnyOrUnknown)
    }

    #[inline]
    pub const fn is_instantiable(self) -> bool {
        self.intersects(Self::Instantiable)
    }

    #[inline]
    pub const fn is_type_with_object_flags(self) -> bool {
        self.intersects(Self::ObjectFlagsType)
    }

    #[inline]
    pub const fn is_non_primitive_union(self) -> bool {
        self.intersects(Self::NonPrimitiveUnion)
    }

    // =========================================================================
    // =============================== INCLUSION ===============================
    // =========================================================================

    #[inline]
    pub const fn includes_wildcard(self) -> bool {
        self.intersects(Self::IncludesWildcard)
    }
    #[inline]
    pub const fn includes_error(self) -> bool {
        self.intersects(Self::IncludesError)
    }

    #[inline]
    pub const fn includes_constrained_type_variable(self) -> bool {
        self.intersects(Self::IncludesConstrainedTypeVariable)
    }

    #[inline]
    pub const fn includes_non_widening_type(self) -> bool {
        self.intersects(Self::IncludesNonWideningType)
    }

    // =========================================================================
    // ============================== TYPE MASKS ===============================
    // =========================================================================

    /// Apply [`Self::IncludesMask`] to the flags, masking out non-includes
    /// related flags
    #[inline]
    #[must_use]
    pub const fn mask_for_includes(self) -> Self {
        // self & Self::IncludesMask
        self.intersection(Self::IncludesMask)
    }
}

impl ObjectFlags {
    #[inline]
    pub const fn is_constrained_type_variable(self) -> bool {
        self.contains(Self::IsConstrainedTypeVariable)
    }

    // =========================================================================
    // =============================== BUILDERS ================================
    // =========================================================================

    /// Conditionally union flags with [`ObjectFlags::PrimitiveUnion`].
    ///
    /// No-op if `is_primitive_union` is `false`.
    #[must_use]
    pub const fn with_primitive_union(self, is_primitive_union: bool) -> Self {
        if is_primitive_union {
            self.union(Self::PrimitiveUnion)
        } else {
            self
        }
    }

    /// Conditionally union flags with [`ObjectFlags::ContainsIntersections`].
    ///
    /// No-op if `contains_intersections` is `false`.
    #[must_use]
    pub const fn with_contains_intersections(self, contains_intersections: bool) -> Self {
        if contains_intersections {
            self.union(Self::ContainsIntersections)
        } else {
            self
        }
    }
}
