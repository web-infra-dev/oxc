use oxc_ast::CommentKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use regex::Regex;

use crate::{context::LintContext, fixer::Fix, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
pub enum BanTsCommentDiagnostic {
    #[error(
        "typescript-eslint(ban-ts-comment): Do not use @ts-{0} because it alters compilation errors."
    )]
    #[diagnostic(severity(warning))]
    Comment(String, #[label] Span),

    #[error(
        "typescript-eslint(ban-ts-comment): Use \"@ts-expect-error\" instead of @ts-ignore, as \"@ts-ignore\" will do nothing if the following line is error-free."
    )]
    #[diagnostic(severity(warning), help("Replace \"@ts-ignore\" with \"@ts-expect-error\"."))]
    IgnoreInsteadOfExpectError(String, #[label] Span),

    #[error("typescript-eslint(ban-ts-comment): Include a description after the @ts-{0} directive to explain why the @ts-{0} is necessary. The description must be {1} characters or longer.")]
    #[diagnostic(severity(warning))]
    CommentRequiresDescription(String, u64, #[label] Span),

    #[error("typescript-eslint(ban-ts-comment): The description for the @ts-{0} directive must match the {1} format.")]
    #[diagnostic(severity(warning))]
    CommentDescriptionNotMatchPattern(String, String, #[label] Span),
}

#[derive(Debug, Default, Clone)]
pub struct BanTsComment(Box<BanTsCommentConfig>);

#[derive(Debug, Clone)]
pub struct BanTsCommentConfig {
    ts_expect_error: DirectiveConfig,
    ts_ignore: DirectiveConfig,
    ts_nocheck: DirectiveConfig,
    ts_check: DirectiveConfig,
    minimum_description_length: u64,
}

impl std::ops::Deref for BanTsComment {
    type Target = BanTsCommentConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for BanTsCommentConfig {
    fn default() -> Self {
        Self {
            ts_expect_error: DirectiveConfig::RequireDescription,
            ts_ignore: DirectiveConfig::Boolean(true),
            ts_nocheck: DirectiveConfig::Boolean(true),
            ts_check: DirectiveConfig::Boolean(false),
            minimum_description_length: 3,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DirectiveConfig {
    Boolean(bool),
    RequireDescription,
    DescriptionFormat(Option<Regex>),
}

impl DirectiveConfig {
    fn from_json(value: &serde_json::Value) -> Option<Self> {
        match value {
            serde_json::Value::Bool(b) => Some(Self::Boolean(*b)),
            serde_json::Value::String(s) => {
                if s == "allow-with-description" {
                    Some(Self::RequireDescription)
                } else {
                    None
                }
            }
            serde_json::Value::Object(o) => {
                let re = o
                    .get("descriptionFormat")
                    .and_then(serde_json::Value::as_str)
                    .and_then(|pattern| Regex::new(pattern).ok());
                Some(Self::DescriptionFormat(re))
            }
            _ => None,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// This rule lets you set which directive comments you want to allow in your codebase.
    ///
    /// ### Why is this bad?
    /// Using TypeScript directives to suppress TypeScript compiler errors
    /// reduces the effectiveness of TypeScript overall.
    ///
    /// ### Example
    /// ```javascript
    /// if (false) {
    ///   // @ts-ignore: Unreachable code error
    ///   console.log('hello');
    /// }
    /// ```
    BanTsComment,
    pedantic
);

impl Rule for BanTsComment {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(BanTsCommentConfig {
            ts_expect_error: value
                .get(0)
                .and_then(|x| x.get("ts-expect-error"))
                .and_then(DirectiveConfig::from_json)
                .unwrap_or(DirectiveConfig::RequireDescription),
            ts_ignore: value
                .get(0)
                .and_then(|x| x.get("ts-ignore"))
                .and_then(DirectiveConfig::from_json)
                .unwrap_or(DirectiveConfig::Boolean(true)),
            ts_nocheck: value
                .get(0)
                .and_then(|x| x.get("ts-nocheck"))
                .and_then(DirectiveConfig::from_json)
                .unwrap_or(DirectiveConfig::Boolean(true)),
            ts_check: value
                .get(0)
                .and_then(|x| x.get("ts-check"))
                .and_then(DirectiveConfig::from_json)
                .unwrap_or(DirectiveConfig::Boolean(false)),
            minimum_description_length: value
                .get(0)
                .and_then(|x| x.get("minimumDescriptionLength"))
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(3),
        }))
    }

    fn run_once(&self, ctx: &LintContext) {
        let comments = ctx.semantic().trivias().comments();
        for (kind, span) in comments {
            let raw = span.source_text(ctx.semantic().source_text());
            if let Some(captures) = find_ts_comment_directive(raw, kind.is_single_line()) {
                // safe to unwrap, if capture success, it can always capture one of the four directives
                let (directive, description) = (captures.0, captures.1);
                if CommentKind::MultiLine == kind
                    && (directive == "check" || directive == "nocheck")
                {
                    continue;
                }

                if raw.trim_start().starts_with('/')
                    && (directive == "check" || directive == "nocheck")
                {
                    continue;
                }

                match self.option(directive) {
                    DirectiveConfig::Boolean(on) => {
                        if *on {
                            if directive == "ignore" {
                                ctx.diagnostic_with_fix(
                                    BanTsCommentDiagnostic::IgnoreInsteadOfExpectError(
                                        directive.to_string(),
                                        span,
                                    ),
                                    || {
                                        Fix::new(
                                            raw.replace("@ts-ignore", "@ts-expect-error"),
                                            span,
                                        )
                                    },
                                );
                            } else {
                                ctx.diagnostic(BanTsCommentDiagnostic::Comment(
                                    directive.to_string(),
                                    span,
                                ));
                            }
                        }
                    }
                    config => {
                        let description_len = description.trim().len();
                        if (description_len as u64) < self.minimum_description_length {
                            ctx.diagnostic(BanTsCommentDiagnostic::CommentRequiresDescription(
                                directive.to_string(),
                                self.minimum_description_length,
                                span,
                            ));
                        }

                        if let DirectiveConfig::DescriptionFormat(Some(ref re)) = config {
                            if !re.is_match(description) {
                                ctx.diagnostic(
                                    BanTsCommentDiagnostic::CommentDescriptionNotMatchPattern(
                                        directive.to_string(),
                                        re.to_string(),
                                        span,
                                    ),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

impl BanTsComment {
    /// get the option for a given directive, caller should guarantee
    /// the directive should be one of the ignore/check/nocheck/expect-error
    fn option(&self, directive: &str) -> &DirectiveConfig {
        match directive {
            "ignore" => &self.ts_ignore,
            "check" => &self.ts_check,
            "nocheck" => &self.ts_nocheck,
            "expect-error" => &self.ts_expect_error,
            _ => {
                unreachable!(
                    "Expected one of ignore/check/nocheck/expect-error, got {}.",
                    directive,
                );
            }
        }
    }
}

pub fn find_ts_comment_directive(raw: &str, single_line: bool) -> Option<(&str, &str)> {
    let prefix = "@ts-";

    let mut last_line_start = None;
    let mut char_indices = raw.char_indices().peekable();
    while let Some((_, c)) = char_indices.next() {
        if c == '\n' {
            last_line_start = char_indices.peek().map(|(i, _)| *i);
        }
    }

    let multi_len = last_line_start.unwrap_or(0);
    let line = &raw[multi_len..];

    // Check the content before the prefix
    let index = line.find(prefix)?;
    if !line[..index]
        .chars()
        .all(|c| c.is_whitespace() || if single_line { c == '/' } else { c == '*' || c == '/' })
    {
        return None;
    }

    let start = index + prefix.len();
    for directive in ["expect-error", "ignore", "nocheck", "check"] {
        if line.get(start..start + directive.len()) == Some(directive) {
            let start = multi_len + index + prefix.len();
            let end = start + directive.len();
            let (directive, description) = (&raw[start..end], &raw[end..]);

            debug_assert!(
                matches!(directive, "expect-error" | "ignore" | "nocheck" | "check"),
                "Expected one of ignore/check/nocheck/expect-error, got {directive}",
            );

            return Some((directive, description));
        }
    }
    None
}

#[test]
fn test() {
    use crate::tester::Tester;
    // A total of 51 test cases passed successfully.
    let pass = vec![
        // ts-expect-error
        ("// just a comment containing @ts-expect-error somewhere", None),
        (r"
            /*
            @ts-expect-error running with long description in a block
            */
		", None),
        (r"
            /* @ts-expect-error not on the last line
            */
        ", None),
        (r"
            /**
             * @ts-expect-error not on the last line
             */
        ", None),
        (r"
            /* not on the last line
            * @ts-expect-error
            */
        ", None),
        (r"
            /* @ts-expect-error
            * not on the last line */
        ", None),
        ("// @ts-expect-error", Some(serde_json::json!([{ "ts-expect-error": false }]))),
        ("// @ts-expect-error here is why the error is expected", Some(serde_json::json!([{"ts-expect-error": "allow-with-description"},]))),
        (r"
            /*
            * @ts-expect-error here is why the error is expected */
        ", Some(serde_json::json!([{"ts-expect-error": "allow-with-description"},]))),
        ("// @ts-expect-error exactly 21 characters", Some(serde_json::json!([
          {
            "ts-expect-error": "allow-with-description",
            "minimumDescriptionLength": 21,
          },
        ]))),
        (r"
            /*
            * @ts-expect-error exactly 21 characters*/
        ", Some(serde_json::json!([{
            "ts-expect-error": "allow-with-description",
            "minimumDescriptionLength": 21,
        }]))),
        ("// @ts-expect-error: TS1234 because xyz", Some(serde_json::json!([
            {
                "ts-expect-error": {
                    "descriptionFormat": "^: TS\\d+ because .+$",
                },
                "minimumDescriptionLength" : 10,
            },
       ]))),
       (r"
            /*
            * @ts-expect-error: TS1234 because xyz */
        ", Some(serde_json::json!([
            {
                "ts-expect-error": {
                    "descriptionFormat": "^: TS\\d+ because .+$",
                },
                "minimumDescriptionLength" : 10,
            },
       ]))),
       ("// @ts-expect-error 👨‍👩‍👧‍👦👨‍👩‍👧‍👦👨‍👩‍👧‍👦", Some(serde_json::json!([{ "ts-expect-error": "allow-with-description" }]))),
        // ts-ignore
       ("// just a comment containing @ts-ignore somewhere", None),
       ("// @ts-ignore", Some(serde_json::json!([{ "ts-ignore": false}]))),
       ("// @ts-ignore I think that I am exempted from any need to follow the rules!", Some(serde_json::json!([{ "ts-ignore": "allow-with-description" }]))),
       (r"
         /*
          @ts-ignore running with long description in a block
         */
		", Some(serde_json::json!([
            {
                "ts-ignore": "allow-with-description",
                "minimumDescriptionLength": 21,
            },
        ]))),
        (r"
            /*
             @ts-ignore
            */
        ", None),
        (r"
            /* @ts-ignore not on the last line
            */
        ", None),
        (r"
            /**
             * @ts-ignore not on the last line
             */
        ", None),
        (r"
            /* not on the last line
            * @ts-expect-error
            */
        ", None),
        (r"
            /* @ts-ignore
            * not on the last line */
        ", None),
        ("// @ts-ignore: TS1234 because xyz", Some(serde_json::json!([
            {
                "ts-ignore": {
                    "descriptionFormat": "^: TS\\d+ because .+$",
                },
                "minimumDescriptionLength": 10,
            },
        ]))),
        ("// @ts-ignore 👨‍👩‍👧‍👦👨‍👩‍👧‍👦👨‍👩‍👧‍👦", Some(serde_json::json!([
            {
                "ts-ignore": "allow-with-description"
            },
        ]))),
        (r"
            /*
            * @ts-ignore here is why the error is expected */
        ", Some(serde_json::json!([
            {
                "ts-ignore": "allow-with-description"
            },
        ]))),
        ("// @ts-ignore exactly 21 characters", Some(serde_json::json!([
            {
                "ts-ignore": "allow-with-description",
                "minimumDescriptionLength": 21,
            },
        ]))),
        (r"
            /*
            * @ts-ignore exactly 21 characters*/
        ", Some(serde_json::json!([
            {
                "ts-ignore": "allow-with-description",
                "minimumDescriptionLength": 21,
            },
        ]))),
        (r"
            /*
            * @ts-ignore: TS1234 because xyz */
        ", Some(serde_json::json!([
            {
                "ts-ignore": {
                    "descriptionFormat": "^: TS\\d+ because .+$",
                },
                "minimumDescriptionLength": 10,
            },
        ]))),
        // ts-nocheck
        ("// just a comment containing @ts-nocheck somewhere", None),
        ("// @ts-nocheck", Some(serde_json::json!([{ "ts-nocheck": false}]))),
        ("// @ts-nocheck no doubt, people will put nonsense here from time to time just to get the rule to stop reporting, perhaps even long messages with other nonsense in them like other // @ts-nocheck or // @ts-ignore things", Some(serde_json::json!([{ "ts-nocheck": "allow-with-description" }]))),
        (r"
        /*
            @ts-nocheck running with long description in a block
        */",
        Some(serde_json::json!([
            {
            "ts-nocheck": "allow-with-description",
            "minimumDescriptionLength": 21,
            },
        ]))),
        ("// @ts-nocheck: TS1234 because xyz", Some(serde_json::json!([
            {
            "ts-nocheck": {
                "descriptionFormat": "^: TS\\d+ because .+$",
            },
            "minimumDescriptionLength": 10,
            },
        ]))),
        ("// @ts-nocheck 👨‍👩‍👧‍👦👨‍👩‍👧‍👦👨‍👩‍👧‍👦", Some(serde_json::json!([
            {
                "ts-nocheck": "allow-with-description",
            },
        ]))),
        ("//// @ts-nocheck - pragma comments may contain 2 or 3 leading slashes", None),
        (r"
            /**
             @ts-nocheck
            */
        ", None),
        (r"
            /*
             @ts-nocheck
            */
        ", None),
        ("/** @ts-nocheck */", None),
        ("/* @ts-nocheck */", None),
        // ts-check
        ("// just a comment containing @ts-check somewhere", None),
        (r"
        /*
            @ts-check running with long description in a block
        */
        ", None),
        ("// @ts-check", Some(serde_json::json!([{ "ts-check": false}]))),
        ("// @ts-check with a description and also with a no-op // @ts-ignore", Some(serde_json::json!([
            {"ts-check": "allow-with-description", "minimumDescriptionLength": 3 },
        ]))),
        ("// @ts-check: TS1234 because xyz", Some(serde_json::json!([
            {
            "ts-check": {
                "descriptionFormat": "^: TS\\d+ because .+$",
            },
            "minimumDescriptionLength": 10,
            },
        ]))),
        ("// @ts-check 👨‍👩‍👧‍👦👨‍👩‍👧‍👦👨‍👩‍👧‍👦", Some(serde_json::json!([
            {
                "ts-check": "allow-with-description",
            },
        ]))),
        ("//// @ts-check - pragma comments may contain 2 or 3 leading slashes", Some(serde_json::json!([
            {
                "ts-check": true,
            },
        ]))),
        (r"
            /**
             @ts-check
            */
        ", Some(serde_json::json!([
            {
                "ts-check": true,
            },
        ]))),
        (r"
            /*
             @ts-check
            */
        ", Some(serde_json::json!([
            {
                "ts-check": true,
            },
        ]))),
        ("/** @ts-check */", Some(serde_json::json!([
            {
                "ts-check": true,
            },
        ]))),
        ("/* @ts-check */", Some(serde_json::json!([
            {
                "ts-check": true,
            },
        ]))),
    ];

    // A total of 57 test cases failed.
    let fail = vec![
        // ts-expect-error
        ("// @ts-expect-error", Some(serde_json::json!([{ "ts-expect-error": true }]))),
        ("/* @ts-expect-error */", Some(serde_json::json!([{ "ts-expect-error": true}]))),
        (
            r"
/*
 @ts-expect-error */
        ",
            Some(serde_json::json!([{ "ts-expect-error": true}])),
        ),
        (
            r"
/** on the last line
 @ts-expect-error */
        ",
            Some(serde_json::json!([{ "ts-expect-error": true}])),
        ),
        (
            r"
/** on the last line
 * @ts-expect-error */
        ",
            Some(serde_json::json!([{ "ts-expect-error": true}])),
        ),
        (
            r"
/**
 * @ts-expect-error: TODO */
        ",
            Some(
                serde_json::json!([{ "ts-expect-error": "allow-with-description", "minimumDescriptionLength": 10}]),
            ),
        ),
        (
            r"
/**
 * @ts-expect-error: TS1234 because xyz */
        ",
            Some(serde_json::json!([{
            "ts-expect-error": {
                "descriptionFormat": "^: TS\\d+ because .+$",
              },
              "minimumDescriptionLength": 25
            }])),
        ),
        (
            r"
/**
 * @ts-expect-error: TS1234 */
        ",
            Some(serde_json::json!([{
            "ts-expect-error": {
                "descriptionFormat": "^: TS\\d+ because .+$",
              },
            }])),
        ),
        (
            r"
/**
 * @ts-expect-error    : TS1234 */
        ",
            Some(serde_json::json!([{
            "ts-expect-error": {
                "descriptionFormat": "^: TS\\d+ because .+$",
              },
            }])),
        ),
        ("/** @ts-expect-error */", Some(serde_json::json!([{ "ts-expect-error": true}]))),
        (
            "// @ts-expect-error: Suppress next line",
            Some(serde_json::json!([{ "ts-expect-error": true}])),
        ),
        (
            "/////@ts-expect-error: Suppress next line",
            Some(serde_json::json!([{ "ts-expect-error": true}])),
        ),
        (
            r"
if (false) {
    // @ts-expect-error: Unreachable code error
    console.log('hello');
}
          ",
            Some(serde_json::json!([{ "ts-expect-error": true}])),
        ),
        (
            "// @ts-expect-error",
            Some(serde_json::json!([
              {
                "ts-expect-error": "allow-with-description",
              },
            ])),
        ),
        (
            "// @ts-expect-error: TODO",
            Some(serde_json::json!([
              {
                "ts-expect-error": "allow-with-description",
                "minimumDescriptionLength": 10,
              },
            ])),
        ),
        (
            "// @ts-expect-error: TS1234 because xyz",
            Some(serde_json::json!([
              {
                "ts-expect-error": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
               "minimumDescriptionLength": 25,
              },
            ])),
        ),
        (
            "// @ts-expect-error: TS1234",
            Some(serde_json::json!([
              {
                "ts-expect-error": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
              },
            ])),
        ),
        (
            "// @ts-expect-error    : TS1234 because xyz",
            Some(serde_json::json!([
              {
                "ts-expect-error": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
              },
            ])),
        ),
        // ts-ignore
        (
            "// @ts-ignore",
            Some(serde_json::json!([{ "ts-ignore": true, "ts-expect-error": true }])),
        ),
        (
            "// @ts-ignore",
            Some(
                serde_json::json!([{ "ts-ignore": true, "ts-expect-error": "allow-with-description" }]),
            ),
        ),
        ("// @ts-ignore", None),
        ("/* @ts-ignore */", Some(serde_json::json!([{ "ts-ignore": true}]))),
        (
            r"
/*
 @ts-ignore */
            ",
            Some(serde_json::json!([{ "ts-ignore": true}])),
        ),
        (
            r"
/** on the last line
 @ts-ignore */
            ",
            Some(serde_json::json!([{ "ts-ignore": true}])),
        ),
        (
            r"
/** on the last line
 * @ts-ignore */
            ",
            Some(serde_json::json!([{ "ts-ignore": true}])),
        ),
        (
            "/** @ts-ignore */",
            Some(serde_json::json!([{ "ts-ignore": true, "ts-expect-error": false }])),
        ),
        (
            r"
/**
 * @ts-ignore: TODO */
            ",
            Some(
                serde_json::json!([{ "ts-expect-error": "allow-with-description", "minimumDescriptionLength": 10 }]),
            ),
        ),
        (
            r"
/**
 * @ts-ignore: TS1234 because xyz */
            ",
            Some(serde_json::json!([{
                "ts-expect-error": {
                    "descriptionFormat": "^: TS\\d+ because .+$",
                  },
                  "minimumDescriptionLength": 25
            }])),
        ),
        ("// @ts-ignore: Suppress next line", None),
        ("/////@ts-ignore: Suppress next line", None),
        (
            r"
if (false) {
    // @ts-ignore: Unreachable code error
    console.log('hello');
}
            ",
            None,
        ),
        ("// @ts-ignore", Some(serde_json::json!([{ "ts-ignore": "allow-with-description" }]))),
        (
            "// @ts-ignore         ",
            Some(serde_json::json!([{ "ts-ignore": "allow-with-description" }])),
        ),
        (
            "// @ts-ignore    .",
            Some(serde_json::json!([{ "ts-ignore": "allow-with-description" }])),
        ),
        (
            "// @ts-ignore: TS1234 because xyz",
            Some(serde_json::json!([
              {
                "ts-ignore": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
               "minimumDescriptionLength": 25,
              },
            ])),
        ),
        (
            "// @ts-ignore: TS1234",
            Some(serde_json::json!([
              {
                "ts-ignore": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
              },
            ])),
        ),
        (
            "// @ts-ignore    : TS1234 because xyz",
            Some(serde_json::json!([
              {
                "ts-ignore": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
              },
            ])),
        ),
        // ts-nocheck
        ("// @ts-nocheck", Some(serde_json::json!([{ "ts-nocheck": true}]))),
        ("// @ts-nocheck", None),
        ("// @ts-nocheck: Suppress next line", None),
        (
            r"
if (false) {
    // @ts-nocheck: Unreachable code error
    console.log('hello');
}
            ",
            None,
        ),
        ("// @ts-nocheck", Some(serde_json::json!([{ "ts-nocheck": "allow-with-description" }]))),
        (
            "// @ts-nocheck: TS1234 because xyz",
            Some(serde_json::json!([
              {
                "ts-nocheck": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
               "minimumDescriptionLength": 25,
              },
            ])),
        ),
        (
            "// @ts-nocheck: TS1234",
            Some(serde_json::json!([
              {
                "ts-nocheck": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
              },
            ])),
        ),
        (
            "// @ts-nocheck    : TS1234 because xyz",
            Some(serde_json::json!([
              {
                "ts-nocheck": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
              },
            ])),
        ),
        // ts-check
        ("// @ts-check", Some(serde_json::json!([{ "ts-check": true}]))),
        ("// @ts-check: Suppress next line", Some(serde_json::json!([{ "ts-check":true}]))),
        (
            r"
if (false) {
    // @ts-check: Unreachable code error
    console.log('hello');
}
            ",
            Some(serde_json::json!([{ "ts-check":true}])),
        ),
        ("// @ts-check", Some(serde_json::json!([{ "ts-check": "allow-with-description" }]))),
        (
            "// @ts-check: TS1234 because xyz",
            Some(serde_json::json!([
              {
                "ts-check": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
               "minimumDescriptionLength" : 25,
              },
            ])),
        ),
        (
            "// @ts-check: TS1234",
            Some(serde_json::json!([
              {
                "ts-check": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
              },
            ])),
        ),
        (
            "// @ts-check    : TS1234 because xyz",
            Some(serde_json::json!([
              {
                "ts-check": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
              },
            ])),
        ),
    ];

    Tester::new(BanTsComment::NAME, pass, fail).test_and_snapshot();
}
