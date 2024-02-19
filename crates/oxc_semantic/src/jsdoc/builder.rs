use std::collections::BTreeMap;
use std::rc::Rc;

use oxc_ast::{AstKind, TriviasMap};
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

use super::{JSDoc, JSDocComment};

pub struct JSDocBuilder<'a> {
    source_text: &'a str,
    trivias: Rc<TriviasMap>,
    docs: BTreeMap<Span, Vec<JSDocComment<'a>>>,
    leading_comments_seen: FxHashSet<u32>,
}

impl<'a> JSDocBuilder<'a> {
    pub fn new(source_text: &'a str, trivias: &Rc<TriviasMap>) -> Self {
        Self {
            source_text,
            trivias: Rc::clone(trivias),
            docs: BTreeMap::default(),
            leading_comments_seen: FxHashSet::default(),
        }
    }

    pub fn build(self) -> JSDoc<'a> {
        JSDoc::new(self.docs)
    }

    pub fn retrieve_jsdoc_comments(&mut self, kind: &AstKind<'a>) -> bool {
        // We may need to expand this kinds for our usecases
        // e.g. TypeScript's `canHaveJSDoc()` function defines so many kinds for their usecases
        // https://github.com/microsoft/TypeScript/blob/d04e3489b0d8e6bc9a8a9396a633632a5a467328/src/compiler/utilities.ts#L4195
        if !(kind.is_statement()
            || kind.is_declaration()
            || matches!(kind, AstKind::ParenthesizedExpression(_)))
        {
            return false;
        }

        // 1. Retrieve every kind of leading comments for this node
        let span = kind.span();
        let mut leading_comments = vec![];
        for (start, comment) in self.trivias.comments().range(..span.start) {
            if !self.leading_comments_seen.contains(start) {
                leading_comments.push((start, comment));
            }
            self.leading_comments_seen.insert(*start);
        }

        // 2. Parse JSDoc comments only
        let leading_jsdoc_comments = leading_comments
            .iter()
            .filter(|(_, comment)| comment.is_multi_line())
            .filter_map(|(start, comment)| {
                let comment_span = Span::new(**start, comment.end());
                // Inside of marker: /*_CONTENT_*/
                let comment_content = comment_span.source_text(self.source_text);
                // Should start with "*": /**_CONTENT_*/
                if !comment_content.starts_with('*') {
                    return None;
                }
                Some(comment_content)
            })
            .map(|comment_content| {
                // Remove the very first `*`?
                // Remove the first `*` and whitespaces in each line?
                JSDocComment::new(comment_content)
            })
            .collect::<Vec<_>>();

        // 3. Save and return `true` to mark JSDoc flag
        if !leading_jsdoc_comments.is_empty() {
            self.docs.insert(span, leading_jsdoc_comments);
            return true;
        }

        false
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::{SourceType, Span};

    use crate::{jsdoc::JSDocComment, Semantic, SemanticBuilder};

    fn build_semantic<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: Option<SourceType>,
    ) -> Semantic<'a> {
        let source_type = source_type.unwrap_or_default();
        let ret = Parser::new(allocator, source_text, source_type).parse();
        let program = allocator.alloc(ret.program);
        let semantic = SemanticBuilder::new(source_text, source_type)
            .with_trivias(ret.trivias)
            .build(program)
            .semantic;
        semantic
    }

    #[allow(clippy::cast_possible_truncation)]
    fn get_jsdoc<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        symbol: &'a str,
        source_type: Option<SourceType>,
    ) -> Option<Vec<JSDocComment<'a>>> {
        let semantic = build_semantic(allocator, source_text, source_type);
        let start = source_text.find(symbol).unwrap() as u32;
        let span = Span::new(start, start + symbol.len() as u32);
        semantic.jsdoc().get_by_span(span)
    }

    fn test_jsdoc_found(source_text: &str, symbol: &str, source_type: Option<SourceType>) {
        let allocator = Allocator::default();
        assert!(
            get_jsdoc(&allocator, source_text, symbol, source_type).is_some(),
            "{symbol} not found in {source_text}"
        );
    }

    fn test_jsdoc_not_found(source_text: &str, symbol: &str) {
        let allocator = Allocator::default();
        assert!(
            get_jsdoc(&allocator, source_text, symbol, None).is_none(),
            "{symbol} found in {source_text}"
        );
    }

    #[test]
    fn not_found() {
        let source_texts = [
            "function foo() {}",
            "// test
            function foo() {}",
            "/* test */function foo() {}",
            "/** test */ ; function foo() {}",
            "/** test */ function foo1() {} function foo() {}",
            "function foo() {} /** test */",
        ];
        for source_text in source_texts {
            test_jsdoc_not_found(source_text, "function foo() {}");
        }
    }

    #[test]
    fn found() {
        let source_texts = [
            "/** test */function foo() {}",
            "/*** test */function foo() {}",
            "
            /** test */
        function foo() {}",
            "/** test */
                function foo() {}",
            "/**
             * test
             * */
            function foo() {}",
            "/** test */
            function foo() {}",
            "/** test */
            // noop
            function foo() {}",
            "/** test */
            /*noop*/
            function foo() {}",
            "/** foo1 */ function foo1() {} /** test */ function foo() {}",
        ];
        for source_text in source_texts {
            test_jsdoc_found(source_text, "function foo() {}", None);
        }
    }

    #[test]
    fn found_on_property_definition() {
        let source = "class Foo {
            /** jsdoc */
            bar: string;
        }";
        let source_type = SourceType::default().with_typescript(true);
        test_jsdoc_found(source, "bar: string;", Some(source_type));
    }

    #[test]
    fn get_all_jsdoc() {
        let allocator = Allocator::default();
        let semantic = build_semantic(
            &allocator,
            r"
            /** 1. ; */
            ;
            /** 2. class X {} *//** 3. class X {} */
            class X {
                /** 4. foo */
                foo = /** 5. (1) */ (1);

                /** THIS is ignored in TS too */
                // bar() {
                //     // ...
                // }
            }

            /** THIS is ignored for now(bound to EOF token in TS) */
            ",
            Some(SourceType::default()),
        );
        assert_eq!(semantic.jsdoc().iter_all().count(), 5);
    }
}
