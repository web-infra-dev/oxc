use bpaf::{Bpaf, Parser};

mod analyse;
mod codegen;
mod derives;
mod generators;
mod logger;
mod output;
mod schema;

use codegen::{Codegen, Runner};
use derives::Derive;
use generators::Generator;
use logger::{log, log_failed, log_result, log_success};
use output::Output;
use schema::Schema;

static SOURCE_PATHS: &[&str] = &[
    "crates/oxc_ast/src/ast/literal.rs",
    "crates/oxc_ast/src/ast/js.rs",
    "crates/oxc_ast/src/ast/ts.rs",
    "crates/oxc_ast/src/ast/jsx.rs",
    "crates/oxc_ast/src/ast/comment.rs",
    "crates/oxc_syntax/src/number.rs",
    "crates/oxc_syntax/src/operator.rs",
    "crates/oxc_span/src/span/types.rs",
    "crates/oxc_span/src/source_type/mod.rs",
    "crates/oxc_regular_expression/src/ast.rs",
];

const AST_CRATE: &str = "crates/oxc_ast";
const TYPESCRIPT_PACKAGE: &str = "npm/oxc-types";
#[expect(dead_code)]
const GITHUB_WATCH_LIST_PATH: &str = ".github/.generated_ast_watch_list.yml";

const DERIVES: &[&dyn Derive] = &[
    &derives::DeriveCloneIn,
    &derives::DeriveGetAddress,
    &derives::DeriveGetSpan,
    &derives::DeriveGetSpanMut,
    &derives::DeriveContentEq,
    &derives::DeriveESTree,
];

const GENERATORS: &[&dyn Generator] = &[
    &generators::AssertLayouts,
    &generators::AstKindGenerator,
    &generators::AstBuilderGenerator,
    &generators::GetIdGenerator,
    &generators::VisitGenerator,
    &generators::VisitMutGenerator,
    &generators::TypescriptGenerator,
];

type Result<R> = std::result::Result<R, String>;

#[derive(Debug, Bpaf)]
struct CliOptions {
    /// Runs all generators but won't write anything down.
    #[bpaf(switch)]
    #[expect(dead_code)]
    dry_run: bool,
    /// Prints no logs.
    quiet: bool,
}

fn main() {
    let cli_options = cli_options().run();

    if cli_options.quiet {
        logger::quiet();
    }

    let codegen = Codegen::new();
    let schema = analyse::analyse(SOURCE_PATHS, &codegen);

    let mut outputs = vec![];

    for generator in GENERATORS {
        let this_outputs = generator.output(&schema).unwrap();
        outputs.extend(this_outputs);
    }

    for generator in DERIVES {
        let this_outputs = generator.output(&schema, &codegen).unwrap();
        outputs.extend(this_outputs);
    }
}
