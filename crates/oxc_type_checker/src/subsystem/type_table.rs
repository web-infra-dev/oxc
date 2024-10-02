use crate::ast::Type;

use oxc_allocator::Vec;
use oxc_index::IndexVec;
use oxc_syntax::{
    symbol::SymbolId,
    types::{TypeFlags, TypeId},
};
// use oxc_type_ast::Type;

#[must_use]
#[derive(Debug, Default)]
pub struct TypeTable<'a> {
    types: IndexVec<TypeId, Type<'a>>,
    flags: IndexVec<TypeId, TypeFlags>,
    /// Symbol associated with each type (if any)
    symbols: IndexVec<TypeId, Option<SymbolId>>,
    /// Type alias associated with each type (if any)
    alias_symbols: IndexVec<TypeId, Option<SymbolId>>,
    alias_type_arguments: IndexVec<TypeId, Option<Vec<'a, TypeId>>>,
    // todo
    /*
    aliasTypeArguments?: readonly Type[]; // Alias type arguments (if any)
    /** @internal */
    permissiveInstantiation?: Type;  // Instantiation with type parameters mapped to wildcard type
    /** @internal */
    restrictiveInstantiation?: Type; // Instantiation with type parameters mapped to unconstrained form
    /** @internal */
    immediateBaseConstraint?: Type;  // Immediate base constraint cache
    */
    /// Widened type cache
    widened: IndexVec<TypeId, Option<TypeId>>,
}

impl<'a> TypeTable<'a> {
    #[inline]
    pub fn get_type(&self, id: TypeId) -> &Type<'a> {
        &self.types[id]
    }

    #[inline]
    pub fn get_flags(&self, id: TypeId) -> TypeFlags {
        self.flags[id]
    }

    #[inline]
    pub fn get_symbol(&self, type_id: TypeId) -> Option<SymbolId> {
        self.symbols[type_id]
    }

    pub fn get_alias_symbol(&self, id: TypeId) -> Option<SymbolId> {
        self.alias_symbols[id]
    }

    pub(crate) fn create_type(
        &mut self,
        r#type: Type<'a>,
        flags: TypeFlags,
        symbol: Option<SymbolId>,
        alias_symbol: Option<SymbolId>,
        alias_type_arguments: Option<Vec<'a, TypeId>>,
    ) -> TypeId {
        let id = self.types.push(r#type);
        let _ = self.flags.push(flags);
        let _ = self.symbols.push(symbol);
        let _ = self.alias_symbols.push(alias_symbol);
        let _ = self.alias_type_arguments.push(alias_type_arguments);
        // TODO
        let _ = self.widened.push(None);
        id
    }
}
