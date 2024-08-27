// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

mod options;

use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
};

use options::OxcOptions;
use oxc::{
    allocator::Allocator,
    ast::{ast::Program, CommentKind, Trivias, Visit},
    codegen::{CodeGenerator, CodegenOptions},
    diagnostics::Error,
    minifier::{CompressOptions, Minifier, MinifierOptions},
    parser::{ParseOptions, Parser},
    semantic::{ScopeFlags, ScopeId, Semantic, SemanticBuilder},
    span::SourceType,
    transformer::{TransformOptions, Transformer},
};
use oxc_index::Idx;
use oxc_linter::Linter;
use oxc_prettier::{Prettier, PrettierOptions};
use serde::Serialize;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct Oxc {
    #[wasm_bindgen(readonly, skip_typescript)]
    #[tsify(type = "Program")]
    pub ast: JsValue,

    #[wasm_bindgen(readonly, skip_typescript)]
    pub ir: String,

    #[wasm_bindgen(readonly, skip_typescript)]
    #[tsify(type = "SymbolTable")]
    pub symbols: JsValue,

    #[wasm_bindgen(readonly, skip_typescript, js_name = "scopeText")]
    pub scope_text: String,

    #[wasm_bindgen(readonly, skip_typescript, js_name = "codegenText")]
    pub codegen_text: String,

    #[wasm_bindgen(readonly, skip_typescript, js_name = "formattedText")]
    pub formatted_text: String,

    #[wasm_bindgen(readonly, skip_typescript, js_name = "prettierFormattedText")]
    pub prettier_formatted_text: String,

    #[wasm_bindgen(readonly, skip_typescript, js_name = "prettierIrText")]
    pub prettier_ir_text: String,

    comments: Vec<Comment>,

    diagnostics: RefCell<Vec<Error>>,

    #[serde(skip)]
    serializer: serde_wasm_bindgen::Serializer,
}

#[derive(Clone, Tsify, Serialize)]
#[tsify(into_wasm_abi)]
pub struct Comment {
    pub r#type: CommentType,
    pub value: String,
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Copy, Tsify, Serialize)]
#[tsify(into_wasm_abi)]
pub enum CommentType {
    Line,
    Block,
}

#[derive(Default, Clone, Serialize)]
pub struct OxcDiagnostic {
    pub start: usize,
    pub end: usize,
    pub severity: String,
    pub message: String,
}

#[wasm_bindgen]
impl Oxc {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self { serializer: serde_wasm_bindgen::Serializer::json_compatible(), ..Self::default() }
    }

    /// Returns Array of String
    /// # Errors
    /// # Panics
    #[wasm_bindgen(js_name = getDiagnostics)]
    pub fn get_diagnostics(&self) -> Result<Vec<JsValue>, serde_wasm_bindgen::Error> {
        Ok(self
            .diagnostics
            .borrow()
            .iter()
            .flat_map(|error| {
                let Some(labels) = error.labels() else { return vec![] };
                labels
                    .map(|label| {
                        OxcDiagnostic {
                            start: label.offset(),
                            end: label.offset() + label.len(),
                            severity: format!("{:?}", error.severity().unwrap_or_default()),
                            message: format!("{error}"),
                        }
                        .serialize(&self.serializer)
                        .unwrap()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>())
    }

    /// Returns comments
    /// # Errors
    #[wasm_bindgen(js_name = getComments)]
    pub fn get_comments(&self) -> Result<Vec<JsValue>, serde_wasm_bindgen::Error> {
        self.comments.iter().map(|c| c.serialize(&self.serializer)).collect()
    }

    /// # Errors
    /// Serde serialization error
    #[wasm_bindgen]
    pub fn run(
        &mut self,
        source_text: &str,
        options: OxcOptions,
    ) -> Result<(), serde_wasm_bindgen::Error> {
        self.diagnostics = RefCell::default();

        let OxcOptions {
            run: run_options,
            parser: parser_options,
            linter: linter_options,
            codegen: codegen_options,
            minifier: minifier_options,
            type_checking: type_checking_options,
        } = options;
        let run_options = run_options.unwrap_or_default();
        let parser_options = parser_options.unwrap_or_default();
        let _linter_options = linter_options.unwrap_or_default();
        let _codegen_options = codegen_options.unwrap_or_default();
        let minifier_options = minifier_options.unwrap_or_default();
        let _type_checking_options = type_checking_options.unwrap_or_default();

        let allocator = Allocator::default();
        // let source_text = &self.source_text;
        let path = PathBuf::from(
            parser_options.source_filename.clone().unwrap_or_else(|| "test.tsx".to_string()),
        );
        let source_type = SourceType::from_path(&path).unwrap_or_default();
        let source_type = match parser_options.source_type.as_deref() {
            Some("script") => source_type.with_script(true),
            Some("module") => source_type.with_module(true),
            _ => source_type,
        };

        let default_parser_options = ParseOptions::default();
        let oxc_parser_options = ParseOptions {
            allow_return_outside_function: parser_options
                .allow_return_outside_function
                .unwrap_or(default_parser_options.allow_return_outside_function),
            preserve_parens: parser_options
                .preserve_parens
                .unwrap_or(default_parser_options.preserve_parens),
            ..default_parser_options
        };
        let ret = Parser::new(&allocator, source_text, source_type)
            .with_options(oxc_parser_options)
            .parse();

        self.comments = Self::map_comments(source_text, &ret.trivias);

        self.save_diagnostics(ret.errors.into_iter().map(Error::from).collect::<Vec<_>>());

        self.ir = format!("{:#?}", ret.program.body);

        let program = allocator.alloc(ret.program);

        let semantic_ret = SemanticBuilder::new(source_text, source_type)
            .with_cfg(true)
            .with_trivias(ret.trivias.clone())
            .with_check_syntax_error(true)
            .build(program);

        if run_options.syntax.unwrap_or_default() {
            self.save_diagnostics(
                semantic_ret.errors.into_iter().map(Error::from).collect::<Vec<_>>(),
            );
        }

        let semantic = Rc::new(semantic_ret.semantic);
        // Only lint if there are not syntax errors
        if run_options.lint.unwrap_or_default() && self.diagnostics.borrow().is_empty() {
            let linter_ret = Linter::default().run(&path, Rc::clone(&semantic));
            let diagnostics = linter_ret.into_iter().map(|e| Error::from(e.error)).collect();
            self.save_diagnostics(diagnostics);
        }

        self.ast = program.serialize(&self.serializer)?;

        if run_options.prettier_format.unwrap_or_default()
            || run_options.prettier_ir.unwrap_or_default()
        {
            let ret = Parser::new(&allocator, source_text, source_type)
                .with_options(ParseOptions { preserve_parens: false, ..oxc_parser_options })
                .parse();

            let mut prettier = Prettier::new(
                &allocator,
                source_text,
                ret.trivias.clone(),
                PrettierOptions::default(),
            );

            if run_options.prettier_format.unwrap_or_default() {
                self.prettier_formatted_text = prettier.build(&ret.program);
            }

            if run_options.prettier_ir.unwrap_or_default() {
                let prettier_doc = prettier.doc(&ret.program).to_string();
                self.prettier_ir_text = {
                    let ret = Parser::new(&allocator, &prettier_doc, SourceType::default()).parse();
                    Prettier::new(
                        &allocator,
                        &prettier_doc,
                        ret.trivias,
                        PrettierOptions::default(),
                    )
                    .build(&ret.program)
                };
            }
        }

        if run_options.transform.unwrap_or_default() {
            let (symbols, scopes) = SemanticBuilder::new(source_text, source_type)
                .build(program)
                .semantic
                .into_symbol_table_and_scope_tree();
            let options = TransformOptions::default();
            let result = Transformer::new(
                &allocator,
                &path,
                source_type,
                source_text,
                ret.trivias.clone(),
                options,
            )
            .build_with_symbols_and_scopes(symbols, scopes, program);
            if !result.errors.is_empty() {
                let errors = result.errors.into_iter().map(Error::from).collect::<Vec<_>>();
                self.save_diagnostics(errors);
            }
        }

        if run_options.scope.unwrap_or_default() || run_options.symbol.unwrap_or_default() {
            let semantic = SemanticBuilder::new(source_text, source_type)
                .build_module_record(PathBuf::new(), program)
                .build(program)
                .semantic;
            if run_options.scope.unwrap_or_default() {
                self.scope_text = Self::get_scope_text(program, &semantic);
            }
            if run_options.symbol.unwrap_or_default() {
                self.symbols = semantic.symbols().serialize(&self.serializer)?;
            }
        }

        let program = allocator.alloc(program);

        if minifier_options.compress.unwrap_or_default()
            || minifier_options.mangle.unwrap_or_default()
        {
            let compress_options = minifier_options.compress_options.unwrap_or_default();
            let options = MinifierOptions {
                mangle: minifier_options.mangle.unwrap_or_default(),
                compress: if minifier_options.compress.unwrap_or_default() {
                    CompressOptions {
                        booleans: compress_options.booleans,
                        drop_console: compress_options.drop_console,
                        drop_debugger: compress_options.drop_debugger,
                        evaluate: compress_options.evaluate,
                        join_vars: compress_options.join_vars,
                        loops: compress_options.loops,
                        typeofs: compress_options.typeofs,
                        ..CompressOptions::default()
                    }
                } else {
                    CompressOptions::all_false()
                },
            };
            Minifier::new(options).build(&allocator, program);
        }

        self.codegen_text = CodeGenerator::new()
            .with_options(CodegenOptions {
                minify: minifier_options.whitespace.unwrap_or_default(),
                ..CodegenOptions::default()
            })
            .build(program)
            .source_text;

        Ok(())
    }

    fn get_scope_text<'a>(program: &Program<'a>, semantic: &Semantic<'a>) -> String {
        struct ScopesTextWriter<'a, 's> {
            semantic: &'s Semantic<'a>,
            scope_text: String,
            indent: usize,
            space: String,
        }

        impl<'a, 's> ScopesTextWriter<'a, 's> {
            fn new(semantic: &'s Semantic<'a>) -> Self {
                Self { semantic, scope_text: String::new(), indent: 0, space: String::new() }
            }

            fn write_line<S: AsRef<str>>(&mut self, line: S) {
                self.scope_text.push_str(&self.space[0..self.indent]);
                self.scope_text.push_str(line.as_ref());
                self.scope_text.push('\n');
            }

            fn indent_in(&mut self) {
                self.indent += 2;
                if self.space.len() < self.indent {
                    self.space.push_str("  ");
                }
            }

            fn indent_out(&mut self) {
                self.indent -= 2;
            }
        }

        impl<'a, 's> Visit<'a> for ScopesTextWriter<'a, 's> {
            fn enter_scope(&mut self, flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
                let scope_id = scope_id.get().unwrap();
                self.write_line(format!("Scope {} ({flags:?}) {{", scope_id.index()));
                self.indent_in();

                let bindings = self.semantic.scopes().get_bindings(scope_id);
                if !bindings.is_empty() {
                    self.write_line("Bindings: {");
                    bindings.iter().for_each(|(name, &symbol_id)| {
                        let symbol_flags = self.semantic.symbols().get_flags(symbol_id);
                        self.write_line(format!("  {name} ({symbol_id:?} {symbol_flags:?})",));
                    });
                    self.write_line("}");
                }
            }

            fn leave_scope(&mut self) {
                self.indent_out();
                self.write_line("}");
            }
        }

        let mut writer = ScopesTextWriter::new(semantic);
        writer.visit_program(program);
        writer.scope_text
    }

    fn save_diagnostics(&self, diagnostics: Vec<Error>) {
        self.diagnostics.borrow_mut().extend(diagnostics);
    }

    fn map_comments(source_text: &str, trivias: &Trivias) -> Vec<Comment> {
        trivias
            .comments()
            .map(|comment| Comment {
                r#type: match comment.kind {
                    CommentKind::SingleLine => CommentType::Line,
                    CommentKind::MultiLine => CommentType::Block,
                },
                value: comment.span.source_text(source_text).to_string(),
                start: comment.span.start,
                end: comment.span.end,
            })
            .collect()
    }
}
