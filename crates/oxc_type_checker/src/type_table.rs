use crate::ast::Type;
use oxc_index::IndexVec;
use oxc_syntax::{
    symbol::SymbolId,
    types::{TypeFlags, TypeId},
};
// use oxc_type_ast::Type;

#[derive(Debug, Default)]
pub struct TypeTable<'a> {
    types: IndexVec<TypeId, Type<'a>>,
    flags: IndexVec<TypeId, TypeFlags>,
    /// Symbol associated with each type (if any)
    symbols: IndexVec<TypeId, Option<SymbolId>>,
    /// Type alias associated with each type (if any)
    alias_symbols: IndexVec<TypeId, Option<SymbolId>>,
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
    pub(crate) fn create_type(
        &mut self,
        r#type: Type<'a>,
        flags: TypeFlags,
        symbol: Option<SymbolId>,
        alias_symbol: Option<SymbolId>,
    ) -> TypeId {
        let id = self.types.push(r#type);
        let _ = self.flags.push(flags);
        let _ = self.symbols.push(symbol);
        let _ = self.alias_symbols.push(alias_symbol);
        // TODO
        let _ = self.widened.push(None);
        id
    }
}
