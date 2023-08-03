use std::sync::Arc;

use itertools::Itertools;
use oxc_allocator::Allocator;
use oxc_diagnostics::{
    miette::{miette, NamedSource},
    Error,
};
extern crate miette;
use oxc_semantic::{Reference, Semantic, SemanticBuilder, SymbolFlags, SymbolId};
use oxc_span::{Atom, SourceType};

pub struct SemanticTester {
    allocator: Allocator,
    source_type: SourceType,
    source_text: &'static str,
    /// SemanticBuilder option
    module_builder: bool,
}

impl SemanticTester {
    /// Create a new tester for a TypeScript test case.
    ///
    /// Use [`SemanticTester::js`] for JavaScript test cases.
    pub fn ts(source_text: &'static str) -> Self {
        Self::new(source_text, SourceType::default().with_module(true).with_typescript(true))
    }

    /// Create a new tester for a JavaScript test case.
    ///
    /// Use [`SemanticTester::ts`] for TypeScript test cases.
    pub fn js(source_text: &'static str) -> Self {
        Self::new(source_text, SourceType::default().with_module(true))
    }

    pub fn new(source_text: &'static str, source_type: SourceType) -> Self {
        Self { allocator: Allocator::default(), source_type, source_text, module_builder: true }
    }

    /// Set the [`SourceType`] to TypeScript (or JavaScript, using `false`)
    #[allow(dead_code)]
    pub fn with_typescript(mut self, yes: bool) -> Self {
        self.source_type = SourceType::default().with_typescript(yes);
        self
    }

    /// Mark the [`SourceType`] as JSX
    #[allow(dead_code)]
    pub fn with_jsx(mut self, yes: bool) -> Self {
        self.source_type = self.source_type.with_jsx(yes);
        self
    }

    /// Set [`SemanticBuilder`]'s `with_module_record_builder` option
    #[allow(dead_code)]
    pub fn with_module_record_builder(mut self, yes: bool) -> Self {
        self.module_builder = yes;
        self
    }

    /// Parse the source text and produce a new [`Semantic`]
    #[allow(unstable_name_collisions)]
    pub fn build(&self) -> Semantic<'_> {
        let parse =
            oxc_parser::Parser::new(&self.allocator, &self.source_text, self.source_type).parse();

        if !parse.errors.is_empty() {
            panic!(
                "\n Failed to parse source:\n{}\n\n{}",
                self.source_text,
                parse
                    .errors
                    .iter()
                    .map(|e| format!("{e}"))
                    .intersperse("\n\n".to_owned())
                    .collect::<String>()
            );
        }

        let program = self.allocator.alloc(parse.program);
        let semantic_ret = SemanticBuilder::new(&self.source_text, self.source_type)
            .with_check_syntax_error(true)
            .with_trivias(&parse.trivias)
            .with_module_record_builder(self.module_builder)
            .build(program);

        if !semantic_ret.errors.is_empty() {
            let report = self.wrap_diagnostics(semantic_ret.errors);
            panic!(
                "Semantic analysis failed:\n\n{}",
                report
                    .iter()
                    .map(|r| r.to_string())
                    .intersperse("\n\n".to_owned())
                    .collect::<String>()
            );
        };

        semantic_ret.semantic
    }

    /// Tests that a symbol with the given name exists at the top-level scope and provides a
    /// wrapper for writing assertions about the found symbol.
    ///
    /// ## Fails
    /// If no symbol with the given name exists at the top-level scope.
    pub fn has_root_symbol(&self, name: &str) -> SymbolTester {
        SymbolTester::new_at_root(self, self.build(), name)
    }

    /// Finds some symbol by name in the source code.
    ///
    /// ## Fails
    /// 1. No symbol with the given name exists,
    /// 2. More than one symbol with the given name exists, so a symbol cannot
    ///    be uniquely obtained.
    pub fn has_some_symbol(&self, name: &str) -> SymbolTester {
        SymbolTester::new_unique(self, self.build(), name)
    }

    fn wrap_diagnostics(&self, diagnostics: Vec<Error>) -> Vec<Error> {
        let name = "test".to_owned()
            + match (self.source_type.is_javascript(), self.source_type.is_jsx()) {
                (true, true) => ".jsx",
                (true, false) => ".js",
                (false, true) => ".tsx",
                (false, false) => ".ts",
            };

        let source = Arc::new(NamedSource::new(name, self.source_text.to_owned()));
        let diagnostics = diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.with_source_code(Arc::clone(&source)))
            .collect();
        diagnostics
    }
}

pub struct SymbolTester<'a> {
    parent: &'a SemanticTester,
    /// Reference to semantic analysis results, from [`SemanticTester`]
    semantic: Semantic<'a>,
    /// Name of the subject symbol
    target: String,
    /// Test case source code.
    ///
    /// Technically, `semantic` also has a reference to the source code with lifetime 'a and this
    /// _could_ also be 'a, but we need it to be 'static to work with miette.
    // source: &'static str,
    /// Symbol data, or error if not found
    data: Result<SymbolId, oxc_diagnostics::Error>,
}

impl<'a> SymbolTester<'a> {
    pub(super) fn new_at_root(
        parent: &'a SemanticTester,
        semantic: Semantic<'a>,
        target: &str,
    ) -> Self {
        let decl =
            semantic.scopes().get_binding(semantic.scopes().root_scope_id(), &Atom::from(target));
        let data = decl.map_or_else(
            || Err(miette!("Could not find declaration for {target}")),
            |decl| Ok(decl),
        );

        SymbolTester { parent, semantic, target: target.to_string(), data }
    }

    pub(super) fn new_unique(
        parent: &'a SemanticTester,
        semantic: Semantic<'a>,
        target: &str,
    ) -> Self {
        let symbols_with_target_name: Vec<_> =
            semantic.scopes().iter_bindings().filter(|(_, _, name)| name == &target).collect();
        let data = match symbols_with_target_name.len() {
            0 => Err(miette!("Could not find declaration for {target}")),
            1 => Ok(symbols_with_target_name.iter().map(|(_, symbol_id, _)| *symbol_id).nth(0).unwrap()),
            n if n > 1 => Err(miette!("Couldn't uniquely resolve symbol id for target {target}; {n} symbols with that name are declared in the source.")),
            _ => unreachable!()
        };

        SymbolTester { parent, semantic, target: target.to_string(), data }
    }

    /// Checks if the resolved symbol contains all flags in `flags`, using [`SymbolFlags::contains()`]
    pub fn contains_flags(mut self, flags: SymbolFlags) -> Self {
        self.data = match self.data {
            Ok(symbol_id) => {
                let found_flags = self.semantic.symbols().get_flag(symbol_id);
                if found_flags.contains(flags) {
                    Ok(symbol_id)
                } else {
                    // let decl_span = self.semantic.symbol_declaration(symbol_id).span();
                    Err(miette!(
                        "Expected {} to contain flags {:?}, but it had {:?}",
                        self.target,
                        flags,
                        found_flags
                    ))
                }
            }
            err => err,
        };
        self
    }

    pub fn intersects_flags(mut self, flags: SymbolFlags) -> Self {
        self.data = match self.data {
            Ok(symbol_id) => {
                let found_flags = self.semantic.symbols().get_flag(symbol_id);
                if found_flags.intersects(flags) {
                    Ok(symbol_id)
                } else {
                    Err(miette!(
                        "Expected {} to intersect with flags {:?}, but it had {:?}",
                        self.target,
                        flags,
                        found_flags
                    ))
                }
            }
            err => err,
        };
        self
    }

    pub fn has_number_of_reads(self, ref_count: usize) -> Self {
        self.has_number_of_references_where(ref_count, |r| r.is_read())
    }

    #[allow(dead_code)]
    pub fn has_number_of_writes(self, ref_count: usize) -> Self {
        self.has_number_of_references_where(ref_count, |r| r.is_write())
    }

    pub fn has_number_of_references(self, ref_count: usize) -> Self {
        self.has_number_of_references_where(ref_count, |_| true)
    }

    pub fn has_number_of_references_where<F>(mut self, ref_count: usize, filter: F) -> Self
    where
        F: FnMut(&Reference) -> bool,
    {
        self.data = match self.data {
            Ok(symbol_id) => {
                let refs = {
                    self.semantic
                        .symbols()
                        .get_resolved_reference_ids(symbol_id)
                        .iter()
                        .map(|r_id| self.semantic.symbols().get_reference(*r_id).clone())
                };
                let num_accepted = refs.filter(filter).count();
                if num_accepted == ref_count {
                    Ok(symbol_id)
                } else {
                    Err(miette!("Expected to find {ref_count} acceptable references, but only found {num_accepted}"))
                }
            }
            e => e,
        };
        self
    }

    pub fn is_exported(mut self) -> Self {
        self.data = match self.data {
            Ok(symbol_id) => {
                let binding = Atom::from(self.target.to_owned());
                if self.semantic.module_record().exported_bindings.contains_key(&binding)
                    && self.semantic.scopes().get_root_binding(&binding) == Some(symbol_id)
                {
                    Ok(symbol_id)
                } else {
                    Err(miette!("Expected {binding} to be exported."))
                }
            }
            e => e,
        };
        self
    }

    /// Complete the test case. Will panic if any of the previously applied
    /// assertions failed.
    pub fn test(self) {
        let res: Result<_, _> = self.into();

        res.unwrap();
    }
}

impl<'a> Into<Result<(), Error>> for SymbolTester<'a> {
    fn into(self) -> Result<(), Error> {
        self.data.map(|_| {}).map_err(|e| e.with_source_code(self.parent.source_text))
    }
}

// fn graphic_hook(diag: dyn Diagnostic) -> GraphicalReportHandler {
//     Box::new(GraphicalReportHandler::new().tab_width(2))
// }
