// // dprint-ignore
// /** @internal */
// export interface SymbolLinks {
//     _symbolLinksBrand: any;
//     immediateTarget?: Symbol;                   // Immediate target of an alias. May be another alias. Do not access directly, use `checker.getImmediateAliasedSymbol` instead.
//     aliasTarget?: Symbol,                       // Resolved (non-alias) target of an alias
//     target?: Symbol;                            // Original version of an instantiated symbol
//     type?: Type;                                // Type of value symbol
//     writeType?: Type;                           // Type of value symbol in write contexts
//     nameType?: Type;                            // Type associated with a late-bound symbol
//     uniqueESSymbolType?: Type;                  // UniqueESSymbol type for a symbol
//     declaredType?: Type;                        // Type of class, interface, enum, type alias, or type parameter
//     typeParameters?: TypeParameter[];           // Type parameters of type alias (undefined if non-generic)
//     instantiations?: Map<string, Type>;         // Instantiations of generic type alias (undefined if non-generic)
//     inferredClassSymbol?: Map<SymbolId, TransientSymbol>; // Symbol of an inferred ES5 constructor function
//     mapper?: TypeMapper;                        // Type mapper for instantiation alias
//     referenced?: boolean;                       // True if alias symbol has been referenced as a value that can be emitted
//     containingType?: UnionOrIntersectionType;   // Containing union or intersection type for synthetic property
//     leftSpread?: Symbol;                        // Left source for synthetic spread property
//     rightSpread?: Symbol;                       // Right source for synthetic spread property
//     syntheticOrigin?: Symbol;                   // For a property on a mapped or spread type, points back to the original property
//     isDiscriminantProperty?: boolean;           // True if discriminant synthetic property
//     resolvedExports?: SymbolTable;              // Resolved exports of module or combined early- and late-bound static members of a class.
//     resolvedMembers?: SymbolTable;              // Combined early- and late-bound members of a symbol
//     exportsChecked?: boolean;                   // True if exports of external module have been checked
//     typeParametersChecked?: boolean;            // True if type parameters of merged class and interface declarations have been checked.
//     isDeclarationWithCollidingName?: boolean;   // True if symbol is block scoped redeclaration
//     bindingElement?: BindingElement;            // Binding element associated with property symbol
//     originatingImport?: ImportDeclaration | ImportCall; // Import declaration which produced the symbol, present if the symbol is marked as uncallable but had call signatures in `resolveESModuleSymbol`
//     lateSymbol?: Symbol;                        // Late-bound symbol for a computed property
//     specifierCache?: Map<ModeAwareCacheKey, string>; // For symbols corresponding to external modules, a cache of incoming path -> module specifier name mappings
//     extendedContainers?: Symbol[];              // Containers (other than the parent) which this symbol is aliased in
//     extendedContainersByFile?: Map<NodeId, Symbol[]>; // Containers (other than the parent) which this symbol is aliased in
//     variances?: VarianceFlags[];                // Alias symbol type argument variance cache
//     deferralConstituents?: Type[];              // Calculated list of constituents for a deferred type
//     deferralWriteConstituents?: Type[];         // Constituents of a deferred `writeType`
//     deferralParent?: Type;                      // Source union/intersection of a deferred type
//     cjsExportMerged?: Symbol;                   // Version of the symbol with all non export= exports merged with the export= target
//     typeOnlyDeclaration?: TypeOnlyAliasDeclaration | false; // First resolved alias declaration that makes the symbol only usable in type constructs
//     typeOnlyExportStarMap?: Map<__String, ExportDeclaration & { readonly isTypeOnly: true, readonly moduleSpecifier: Expression }>; // Set on a module symbol when some of its exports were resolved through a 'export type * from "mod"' declaration
//     typeOnlyExportStarName?: __String;          // Set to the name of the symbol re-exported by an 'export type *' declaration, when different from the symbol name
//     isConstructorDeclaredProperty?: boolean;    // Property declared through 'this.x = ...' assignment in constructor
//     tupleLabelDeclaration?: NamedTupleMember | ParameterDeclaration; // Declaration associated with the tuple's label
//     accessibleChainCache?: Map<string, Symbol[] | undefined>;
//     filteredIndexSymbolCache?: Map<string, Symbol> //Symbol with applicable declarations
//     requestedExternalEmitHelpers?: ExternalEmitHelpers; // External emit helpers already checked for this symbol.
// }

#[derive(Debug, Default)]
pub(crate) struct SymbolLinks {}
