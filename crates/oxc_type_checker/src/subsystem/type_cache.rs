//! Type caches portion of `checker.ts`

use oxc_syntax::types::TypeId;
use rustc_hash::FxHashMap;

#[allow(dead_code)]
#[derive(Debug, Default)]
pub(crate) struct TypeCache {
    /// Tuple types
    ///
    /// ```typescript
    /// var tupleTypes = new Map<string, GenericType>();
    /// ```
    pub tuples: FxHashMap<String, TypeId>,
    /// Union types
    ///
    /// ```typescript
    /// var unionTypes = new Map<string, UnionType>();
    /// ```
    pub unions: FxHashMap<String, TypeId>,
    /// Unions of Union types
    ///
    /// ```typescript
    /// var unionOfUnionTypes = new Map<string, Type>();
    /// ```
    pub union_of_unions: FxHashMap<String, TypeId>,
    // var intersectionTypes = new Map<string, Type>();
    // var stringLiteralTypes = new Map<string, StringLiteralType>();
    // var numberLiteralTypes = new Map<number, NumberLiteralType>();
    // var bigIntLiteralTypes = new Map<string, BigIntLiteralType>();
    // var enumLiteralTypes = new Map<string, LiteralType>();
    // var indexedAccessTypes = new Map<string, IndexedAccessType>();
    // var templateLiteralTypes = new Map<string, TemplateLiteralType>();
    // var stringMappingTypes = new Map<string, StringMappingType>();
    // var substitutionTypes = new Map<string, SubstitutionType>();
    // var subtypeReductionCache = new Map<string, Type[]>();
    // var decoratorContextOverrideTypeCache = new Map<string, Type>();
    // var cachedTypes = new Map<string, Type>();
    // var evolvingArrayTypes: EvolvingArrayType[] = [];
    // var undefinedProperties: SymbolTable = new Map();
    // var markerTypes = new Set<number>();
}
