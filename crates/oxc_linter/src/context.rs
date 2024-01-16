use std::{cell::RefCell, path::Path, rc::Rc, sync::Arc};

use oxc_codegen::{Codegen, CodegenOptions};
use oxc_diagnostics::Error;
use oxc_semantic::{AstNodes, JSDocComment, ScopeTree, Semantic, SymbolTable};
use oxc_span::SourceType;

use crate::{
    disable_directives::DisableDirectives,
    fixer::{Fix, Message},
    AstNode, LintSettings,
};

pub struct LintContext<'a> {
    semantic: Rc<Semantic<'a>>,

    disable_directives: Rc<DisableDirectives<'a>>,

    /// Whether or not to apply code fixes during linting.
    fix: bool,

    current_rule_name: &'static str,

    file_path: Box<Path>,

    settings: Arc<LintSettings>,

    diagnostics: RefCell<Vec<Message<'a>>>,
}

impl<'a> LintContext<'a> {
    pub fn new(file_path: Box<Path>, semantic: &Rc<Semantic<'a>>) -> Self {
        Self {
            semantic: Rc::clone(semantic),
            disable_directives: Rc::new(DisableDirectives::default()),
            fix: false,
            current_rule_name: "",
            file_path,
            settings: Arc::new(LintSettings::default()),
            diagnostics: RefCell::new(vec![]),
        }
    }

    #[must_use]
    pub fn clone_without_diagnostics(&self) -> Self {
        Self {
            semantic: Rc::clone(&self.semantic),
            disable_directives: Rc::clone(&self.disable_directives),
            fix: self.fix,
            current_rule_name: "",
            file_path: self.file_path.clone(),
            settings: Arc::clone(&self.settings),
            diagnostics: RefCell::new(vec![]),
        }
    }

    #[must_use]
    pub fn with_disable_directives(mut self, directives: &Rc<DisableDirectives<'a>>) -> Self {
        self.disable_directives = Rc::clone(directives);
        self
    }

    #[must_use]
    pub fn with_fix(mut self, fix: bool) -> Self {
        self.fix = fix;
        self
    }

    #[must_use]
    pub fn with_settings(mut self, settings: &Arc<LintSettings>) -> Self {
        self.settings = Arc::clone(settings);
        self
    }

    #[must_use]
    pub fn with_rule_name(mut self, name: &'static str) -> Self {
        self.current_rule_name = name;
        self
    }

    pub fn semantic(&self) -> &Rc<Semantic<'a>> {
        &self.semantic
    }

    pub fn disable_directives(&self) -> &DisableDirectives<'a> {
        &self.disable_directives
    }

    pub fn settings(&self) -> &LintSettings {
        &self.settings
    }

    pub fn source_text(&self) -> &'a str {
        self.semantic().source_text()
    }

    pub fn source_type(&self) -> &SourceType {
        self.semantic().source_type()
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /* Diagnostics */

    pub fn into_message(self) -> Vec<Message<'a>> {
        self.diagnostics.into_inner()
    }

    fn add_diagnostic(&self, message: Message<'a>) {
        if !self.disable_directives.contains(self.current_rule_name, message.start()) {
            self.diagnostics.borrow_mut().push(message);
        }
    }

    pub fn diagnostic<T: Into<Error>>(&self, diagnostic: T) {
        self.add_diagnostic(Message::new(diagnostic.into(), None));
    }

    pub fn diagnostic_with_fix<T, F>(&self, diagnostic: T, fix: F)
    where
        T: Into<Error>,
        F: FnOnce() -> Fix<'a>,
    {
        if self.fix {
            self.add_diagnostic(Message::new(diagnostic.into(), Some(fix())));
        } else {
            self.diagnostic(diagnostic);
        }
    }

    pub fn nodes(&self) -> &AstNodes<'a> {
        self.semantic().nodes()
    }

    pub fn scopes(&self) -> &ScopeTree {
        self.semantic().scopes()
    }

    pub fn symbols(&self) -> &SymbolTable {
        self.semantic().symbols()
    }

    #[allow(clippy::unused_self)]
    pub fn codegen(&self) -> Codegen<false> {
        Codegen::<false>::new(0, CodegenOptions)
    }

    /* JSDoc */
    pub fn jsdoc(&self, node: &AstNode<'a>) -> Option<JSDocComment<'a>> {
        self.semantic().jsdoc().get_by_node(node)
    }
}
