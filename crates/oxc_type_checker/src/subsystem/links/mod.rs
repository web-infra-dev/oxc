mod symbol_links;

use oxc_semantic::SymbolId;
use rustc_hash::FxHashMap;
use std::marker::PhantomData;

pub(crate) use symbol_links::SymbolLinks;

#[derive(Debug, Default)]
pub(crate) struct Links<'a> {
    symbol_links: FxHashMap<SymbolId, SymbolLinks>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Links<'a> {
    /// ```typescript
    /// function getSymbolLinks(symbol: Symbol): SymbolLinks {
    ///     if (symbol.flags & SymbolFlags.Transient) return (symbol as TransientSymbol).links;
    ///     const id = getSymbolId(symbol);
    ///     return symbolLinks[id] ??= new SymbolLinks();
    /// }
    /// ```
    pub fn get_symbol_links_mut(&mut self, symbol_id: SymbolId) -> &mut SymbolLinks {
        self.symbol_links.entry(symbol_id).or_default()
    }

    pub fn get_symbol_links(&self, symbol_id: SymbolId) -> Option<&SymbolLinks> {
        self.symbol_links.get(&symbol_id)
    }
}
