use crate::suite::{Case, Suite, TestResult};
use oxc_span::SourceType;
use oxc_tasks_common::project_root;
use std::io::Write;
use std::{
    fs::File,
    path::{Path, PathBuf},
};

static FIXTURES_PATH: &str =
    "tasks/coverage/babel/packages/babel-generator/test/fixtures/sourcemaps";

pub struct SourcemapSuite<T: Case> {
    test_root: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> SourcemapSuite<T> {
    pub fn new() -> Self {
        Self { test_root: project_root().join(FIXTURES_PATH), test_cases: vec![] }
    }
}

impl<T: Case> Suite<T> for SourcemapSuite<T> {
    fn get_test_root(&self) -> &Path {
        &self.test_root
    }

    fn save_test_cases(&mut self, tests: Vec<T>) {
        self.test_cases = tests;
    }

    fn get_test_cases(&self) -> &Vec<T> {
        &self.test_cases
    }

    fn get_test_cases_mut(&mut self) -> &mut Vec<T> {
        &mut self.test_cases
    }

    fn skip_test_path(&self, path: &Path) -> bool {
        let path = path.to_string_lossy();
        !path.contains("input.js")
    }

    fn run_coverage(&self, name: &str, _args: &crate::AppArgs) {
        let path = project_root().join(format!("tasks/coverage/{name}.snap"));
        let mut file = File::create(path).unwrap();

        let mut tests = self.get_test_cases().iter().collect::<Vec<_>>();
        tests.sort_by_key(|case| case.path());

        for case in tests {
            let result = case.test_result();
            let path = case.path().to_string_lossy();
            let result = match result {
                TestResult::Snapshot(snapshot) => snapshot,
                TestResult::ParseError(error, _) => error,
                _ => {
                    unreachable!()
                }
            };
            writeln!(file, "- {path}").unwrap();
            writeln!(file, "{result}\n\n").unwrap();
        }
    }
}

pub struct SourcemapCase {
    path: PathBuf,
    code: String,
    source_type: SourceType,
    result: TestResult,
}

impl SourcemapCase {
    pub fn source_type(&self) -> SourceType {
        self.source_type
    }

    fn create_visualizer_text(
        source: &str,
        output: &str,
        tokens: &[(u32, u32, u32, u32)],
    ) -> String {
        let source_lines = source.lines().collect::<Vec<_>>();
        let output_lines = output.lines().collect::<Vec<_>>();
        let mut s = String::new();

        tokens.iter().reduce(|pre, cur| {
            s.push_str(&format!(
                "({}:{}-{}:{}) {:?}",
                pre.0,
                pre.1,
                cur.0,
                cur.1,
                Self::str_slice(&source_lines, (pre.0, pre.1), (cur.0, cur.1))
            ));
            s.push_str(" --> ");
            s.push_str(&format!(
                "({}:{}-{}:{}) {:?}",
                pre.2,
                pre.3,
                cur.2,
                cur.3,
                Self::str_slice(&output_lines, (pre.2, pre.3), (cur.2, cur.3))
            ));
            s.push('\n');
            cur
        });

        s
    }

    fn str_slice(str: &[&str], start: (u32, u32), end: (u32, u32)) -> String {
        if start.0 == end.0 {
            return str[start.0 as usize][start.1 as usize..end.1 as usize].to_string();
        }

        let mut s = String::new();

        for i in start.0..end.0 {
            if i == start.0 {
                s.push_str(&str[i as usize][start.1 as usize..]);
            } else if i == end.0 {
                s.push_str(&str[i as usize][..end.1 as usize]);
            } else {
                s.push_str(str[i as usize]);
            }
        }

        s
    }
}

impl Case for SourcemapCase {
    fn new(path: PathBuf, code: String) -> Self {
        let source_type = SourceType::from_path(&path).unwrap();
        Self { path, code, source_type, result: TestResult::ToBeRun }
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn test_result(&self) -> &TestResult {
        &self.result
    }

    fn run(&mut self) {
        let source_type = self.source_type();
        self.result = self.execute(source_type);
    }

    fn execute(&mut self, source_type: SourceType) -> TestResult {
        let source_text = self.code();
        let allocator = oxc_allocator::Allocator::default();
        let ret = oxc_parser::Parser::new(&allocator, source_text, source_type).parse();

        if !ret.errors.is_empty() {
            if let Some(error) = ret.errors.into_iter().next() {
                let error = error.with_source_code(source_text.to_string());
                return TestResult::ParseError(error.to_string(), false);
            }
        }

        let codegen_options = oxc_codegen::CodegenOptions::default();
        let mut codegen = oxc_codegen::Codegen::<false>::new(source_text.len(), codegen_options);
        let content = codegen.with_sourcemap(source_text, "").build(&ret.program);
        let map = codegen.into_sourcemap();
        let tokens = map
            .tokens()
            .map(|token| {
                (
                    token.get_src_line(),
                    token.get_src_col(),
                    token.get_dst_line(),
                    token.get_dst_col(),
                )
            })
            .collect::<Vec<_>>();

        let mut result = String::new();
        result.push_str(Self::create_visualizer_text(source_text, &content, &tokens).as_str());

        TestResult::Snapshot(result)
    }
}
