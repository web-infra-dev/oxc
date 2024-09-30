//! Type caches portion of `checker.ts`

use std::{borrow::Borrow, cell::RefCell, ops::Deref};

use oxc_allocator::{Allocator, CloneIn, Vec};
use oxc_syntax::types::TypeId;
use rustc_hash::FxHashMap;

type Cache<K> = RefCell<FxHashMap<K, TypeId>>;

#[allow(dead_code)]
pub(crate) struct TypeCache<'a> {
    alloc: &'a Allocator,
    /// Tuple types
    ///
    /// ```typescript
    /// var tupleTypes = new Map<string, GenericType>();
    /// ```
    tuples: Cache<String>,
    /// Union types
    ///
    /// ```typescript
    /// var unionTypes = new Map<string, UnionType>();
    /// ```
    unions: Cache<TypeList<'a>>,
    /// Unions of Union types
    ///
    /// ```typescript
    /// var unionOfUnionTypes = new Map<string, Type>();
    /// ```
    union_of_unions: Cache<String>,
    /// Intersection types
    ///
    /// ```typescript
    /// var intersectionTypes = new Map<string, Type>();
    /// ```
    intersections: Cache<TypeList<'a>>,
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

impl<'a> TypeCache<'a> {
    pub fn new(alloc: &'a Allocator) -> Self {
        Self {
            alloc,
            tuples: Cache::default(),
            unions: Cache::default(),
            union_of_unions: Cache::default(),
            intersections: Cache::default(),
        }
    }

    pub fn get_union(&self, types: &TypeList<'a>) -> Option<TypeId> {
        self.unions.borrow().get(types).copied()
    }

    pub fn add_union(&self, types: TypeList<'a>, id: TypeId) {
        let existing = self.unions.borrow_mut().insert(types, id);
        debug_assert!(existing.is_none());
    }
}

/// Stable list of types, meant to replace TypeScript's approach to creating
/// unique string ids to index type caches.
///
/// Replacement for `getTypeListId`, which relies on strings to index
/// compound types.
///
/// <details><summary><code>getTypeListId</code> implementation</summary>
///
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
///
/// </details>
///
/// TODO: use Box<[TypeId]> when #6195 is merged
#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct TypeList<'a>(Vec<'a, TypeId>);

impl<'a> TypeList<'a> {
    #[must_use]
    pub fn new(alloc: &'a Allocator, types: &[TypeId]) -> Self {
        let mut v = Vec::with_capacity_in(types.len(), alloc);
        v.copy_from_slice(types);
        v.sort_unstable();
        v.dedup();
        v.shrink_to_fit();
        Self(v)
    }

    #[must_use]
    pub fn from_iter<I>(alloc: &'a Allocator, iter: I) -> Self
    where
        I: IntoIterator<Item = TypeId>,
    {
        let mut v = Vec::from_iter_in(iter, alloc);
        v.sort_unstable();
        v.dedup();
        v.shrink_to_fit();
        Self(v)
    }

    pub fn iter(&self) -> impl Iterator<Item = TypeId> + '_ {
        self.0.iter().copied()
    }
}

impl Deref for TypeList<'_> {
    type Target = [TypeId];

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl Borrow<[TypeId]> for TypeList<'_> {
    fn borrow(&self) -> &[TypeId] {
        self.0.as_ref()
    }
}

impl<'a> CloneIn<'a> for TypeList<'a> {
    type Cloned = TypeList<'a>;

    fn clone_in(&self, alloc: &'a Allocator) -> TypeList<'a> {
        let mut v = Vec::with_capacity_in(self.0.len(), alloc);
        v.copy_from_slice(self.0.as_ref());
        TypeList(v)
    }
}
