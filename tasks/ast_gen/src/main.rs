use bpaf::{Bpaf, Parser};

mod analyse;
mod codegen;
mod derives;
mod generators;
mod logger;
mod output;
mod schema;
mod utils;

use codegen::{Codegen, Runner};
use derives::Derive;
use generators::Generator;
use logger::{log, log_failed, log_result, log_success};
use output::{Output, RawOutput};
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

type Result<R> = std::result::Result<R, ()>;

#[derive(Debug, Bpaf)]
struct CliOptions {
    /// Runs all generators but won't write anything down.
    #[bpaf(switch)]
    dry_run: bool,
    /// Prints no logs.
    quiet: bool,
}

fn main() {
    // Parse CLI options
    let cli_options = cli_options().run();

    // Init logger
    if cli_options.quiet {
        logger::quiet();
    }

    // Parse inputs and generate `Schema`
    let codegen = Codegen::new();
    let mut schema = analyse::analyse(SOURCE_PATHS, &codegen);

    // Run modify actions
    for runner in GENERATORS {
        runner.modify(&mut schema);
    }

    for runner in DERIVES {
        runner.modify(&mut schema);
    }

    // Run generators
    let mut outputs = vec![];

    for runner in GENERATORS {
        if !["GetIdGenerator", "AstKindGenerator", "AssertLayouts"].contains(&runner.name()) {
            continue;
        }

        log!("Generate {}... ", runner.name());
        let result = runner.run(&schema, &codegen);
        log_result!(result);
        let runner_outputs = result.unwrap();
        let runner_path = runner.file_path();
        outputs.extend(runner_outputs.into_iter().map(|output| output.into_raw(runner_path)));
    }

    for runner in DERIVES {
        if ![
            "DeriveCloneIn",
            "DeriveGetSpan",
            "DeriveGetSpanMut",
            "DeriveContentEq",
            "DeriveGetAddress",
        ]
        .contains(&runner.name())
        {
            continue;
        }

        log!("Derive {}... ", runner.name());
        let result = runner.run(&schema, &codegen);
        log_result!(result);
        let runner_outputs = result.unwrap();
        let runner_path = runner.file_path();
        outputs.extend(runner_outputs.into_iter().map(|output| output.into_raw(runner_path)));
    }

    // Add CI filter file to outputs
    // outputs.push(generate_ci_filter(&outputs));

    // Write outputs to disk
    if !cli_options.dry_run {
        for output in outputs {
            output.write_to_file().unwrap();
        }
    }
}

#[expect(dead_code)]
fn generate_ci_filter(outputs: &[RawOutput]) -> RawOutput {
    log!("Generate CI filter... ");

    let mut code = "src:\n".to_string();
    let mut push_item = |path: &str| code.push_str(format!("  - '{path}'\n").as_str());

    for input in SOURCE_PATHS {
        push_item(input);
    }

    for output in outputs {
        push_item(output.path.as_str());
    }

    push_item("tasks/ast_tools/src/**");
    push_item(GITHUB_WATCH_LIST_PATH);

    log_success!();

    Output::Yaml { path: GITHUB_WATCH_LIST_PATH.to_string(), code }.into_raw(file!())
}
