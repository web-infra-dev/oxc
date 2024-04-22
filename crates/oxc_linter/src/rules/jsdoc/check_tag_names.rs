use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use phf::phf_set;
use serde::Deserialize;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jsdoc(check-tag-names): Invalid tag name found.")]
#[diagnostic(severity(warning), help("{1}"))]
struct CheckTagNamesDiagnostic(#[label] pub Span, String);

#[derive(Debug, Default, Clone)]
pub struct CheckTagNames(Box<CheckTagnamesConfig>);

declare_oxc_lint!(
    /// ### What it does
    /// Reports invalid block tag names.
    /// Additionally checks for tag names that are redundant when using a type checker such as TypeScript.
    ///
    /// ### Why is this bad?
    /// Using invalid tags can lead to confusion and make the documentation harder to read.
    ///
    /// ### Example
    /// ```javascript
    /// // Passing
    /// /** @param */
    ///
    /// // Failing
    /// /** @Param */
    /// /** @foo */
    ///
    /// /**
    ///  * This is redundant when typed.
    ///  * @type {string}
    ///  */
    /// ```
    CheckTagNames,
    correctness
);

#[derive(Debug, Default, Clone, Deserialize)]
struct CheckTagnamesConfig {
    #[serde(default, rename = "definedTags")]
    defined_tags: Vec<String>,
    #[serde(default, rename = "jsxTags")]
    jsx_tags: bool,
    #[serde(default)]
    typed: bool,
}

const VALID_BLOCK_TAGS: phf::Set<&'static str> = phf_set! {
"abstract",
"access",
"alias",
"async",
"augments",
"author",
"borrows",
"callback",
"class",
"classdesc",
"constant",
"constructs",
"copyright",
"default",
"deprecated",
"description",
"enum",
"event",
"example",
"exports",
"external",
"file",
"fires",
"function",
"generator",
"global",
"hideconstructor",
"ignore",
"implements",
"inheritdoc",
"inner",
"instance",
"interface",
"kind",
"lends",
"license",
"listens",
"member",
"memberof",
"memberof!",
"mixes",
"mixin",
// Undocumented, but exists
// https://github.com/jsdoc/jsdoc/blob/a08ac18a11f5b0d93421d1e8ecf632468db2d045/packages/jsdoc-tag/lib/definitions/core.js#L374
"modifies",
"module",
"name",
"namespace",
"override",
"package",
"param",
"private",
"property",
"protected",
"public",
"readonly",
"requires",
"returns",
"see",
"since",
"static",
"summary",
"this",
"throws",
"todo",
"tutorial",
"type",
"typedef",
"variation",
"version",
"yields",
// TypeScript specific --
"import",
"internal",
"overload",
"satisfies",
"template",
};

const JSX_TAGS: phf::Set<&'static str> = phf_set! {
"jsx",
"jsxFrag",
"jsxImportSource",
"jsxRuntime",
};

const ALWAYS_INVALID_TAGS_IF_TYPED: phf::Set<&'static str> = phf_set! {
"augments",
"callback",
"class",
"enum",
"implements",
"private",
"property",
"protected",
"public",
"readonly",
"this",
"type",
"typedef",
};
const OUTSIDE_AMBIENT_INVALID_TAGS_IF_TYPED: phf::Set<&'static str> = phf_set! {
"abstract",
"access",
"class",
"constant",
"constructs",
// I'm not sure but this seems to be allowed...
// https://github.com/gajus/eslint-plugin-jsdoc/blob/e343ab5b1efaa59b07c600138aee070b4083857e/src/rules/checkTagNames.js#L140
// "default",
"enum",
"export",
"exports",
"function",
"global",
"inherits",
"instance",
"interface",
"member",
"memberof",
"memberOf",
"method",
"mixes",
"mixin",
"module",
"name",
"namespace",
"override",
"property",
"requires",
"static",
"this",
};

impl Rule for CheckTagNames {
    fn from_configuration(value: serde_json::Value) -> Self {
        value
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .map_or_else(Self::default, |value| Self(Box::new(value)))
    }

    fn run_once(&self, ctx: &LintContext) {
        let settings = &ctx.settings().jsdoc;
        let config = &self.0;

        let user_defined_tags = settings.list_user_defined_tag_names();

        let is_dts = ctx.file_path().to_str().map_or(false, |p| p.ends_with(".d.ts"));
        // NOTE: Original rule seems to check `declare` context with visiting AST nodes.
        // https://github.com/gajus/eslint-plugin-jsdoc/blob/e343ab5b1efaa59b07c600138aee070b4083857e/src/rules/checkTagNames.js#L121
        // But...
        // - No test case covers this(= only checks inside of `.d.ts`)
        // - I never seen this usage before
        // So, I leave this part out for now.
        let is_declare = false;
        let is_ambient = is_dts || is_declare;

        // TODO: Bundle multiple diagnostics into one?
        for jsdoc in ctx.semantic().jsdoc().iter_all() {
            for tag in jsdoc.tags() {
                let tag_name = tag.kind.parsed();

                // If explicitly blocked, report
                if let Some(reason) = settings.check_blocked_tag_name(tag_name) {
                    ctx.diagnostic(CheckTagNamesDiagnostic(tag.kind.span, reason));
                    continue;
                }

                // If user defined, skip
                if user_defined_tags.contains(&tag_name.to_string()) {
                    continue;
                }

                let is_valid = config.jsx_tags && JSX_TAGS.contains(tag_name)
                    || config.defined_tags.contains(&tag_name.to_string())
                    || VALID_BLOCK_TAGS.contains(tag_name);

                // If invalid or unknown, report
                if !is_valid {
                    ctx.diagnostic(CheckTagNamesDiagnostic(
                        tag.kind.span,
                        format!("`@{tag_name}` is invalid tag name."),
                    ));
                    continue;
                }

                // If valid but preferred, report to use it
                if let Some(reason) = settings.check_preferred_tag_name(tag_name) {
                    ctx.diagnostic(CheckTagNamesDiagnostic(tag.kind.span, reason));
                    continue;
                }

                // Additional check for `typed` mode
                if config.typed {
                    if ALWAYS_INVALID_TAGS_IF_TYPED.contains(tag_name) {
                        ctx.diagnostic(CheckTagNamesDiagnostic(
                            tag.kind.span,
                            format!("`@{tag_name}` is redundant when using a type system."),
                        ));
                        continue;
                    }

                    if tag.kind.parsed() == "template" && tag.comment().parsed().is_empty() {
                        ctx.diagnostic(CheckTagNamesDiagnostic(
                                tag.kind.span,
                                format!("`@{tag_name}` without a name is redundant when using a type system."),
                            ));
                        continue;
                    }

                    if !is_ambient && OUTSIDE_AMBIENT_INVALID_TAGS_IF_TYPED.contains(tag_name) {
                        ctx.diagnostic(CheckTagNamesDiagnostic(
                                tag.kind.span,
                                format!("`@{tag_name}` is redundant outside of ambient(`declare` or `.d.ts`) contexts when using a type system."),
                            ));
                        continue;
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
("
			          /**
			           * @param foo (pass: valid name)
			           */
			          function quux (foo) {
			
			          }
			      ", None, None),
("
			          /**
			           * @memberof! foo (pass: valid name)
			           */
			          function quux (foo) {
			
			          }
			      ", None, None),
("
			          /**
			           * @arg foo (pass: invalid name but user preferred)
			           */
			          function quux (foo) {
			
			          }
			      ", None, Some(serde_json::json!({
        "jsdoc": {
          "tagNamePreference": {
            "param": "arg",
          },
        },
      }))),
("
			          /**
			           * @bar foo (pass: invalid name but defined)
			           */
			          function quux (foo) {
			
			          }
			      ", Some(serde_json::json!([
        {
          "definedTags": [
            "bar",
          ],
        },
      ])), None),
("
			          /**
			           * @baz @bar foo (pass: invalid names but defined)
			           */
			          function quux (foo) {
			
			          }
			      ", Some(serde_json::json!([
        {
          "definedTags": [
            "baz", "bar",
          ],
        },
      ])), None),
("
			          /**
			           * @baz @bar foo (pass: invalid names but user preferred)
			           */
			          function quux (foo) {
			
			          }
			      ", None, Some(serde_json::json!({
        "jsdoc": {
          "tagNamePreference": {
            "param": "baz",
            "returns": {
              "message": "Prefer `bar`",
              "replacement": "bar",
            },
            "todo": false,
          },
        },
      }))),
("
			      /**
			       * @returns (pass: valid name)
			       */
			      function quux (foo) {}
			      ", None, None),
("", None, None),
("
			          /**
			           * (pass: no tag)
			           */
			          function quux (foo) {
			
			          }
			      ", None, None),
("
			          /**
			           * @todo (pass: valid name)
			           */
			          function quux () {
			
			          }
			      ", None, None),
("
			          /**
			           * @extends Foo (pass: invalid name but user preferred)
			           */
			          function quux () {
			
			          }
			      ", None, Some(serde_json::json!({
        "jsdoc": {
          "tagNamePreference": {
            "augments": {
              "message": "@extends is to be used over @augments.",
              "replacement": "extends",
            },
          },
        },
      }))),
("
			          /**
			           * (Set tag name preference to itself to get aliases to
			           *   work along with main tag name.)
			           * @augments Bar
			           * @extends Foo (pass: invalid name but user preferred)
			           */
			          function quux () {
			          }
			      ", None, Some(serde_json::json!({
        "jsdoc": {
          "tagNamePreference": {
            "extends": "extends",
          },
        },
      }))),
("
			      /**
			       * Registers the `target` class as a transient dependency; each time the dependency is resolved a new instance will be created.
			       *
			       * @param target - The class / constructor function to register as transient.
			       *
			       * @example ```ts
			      @transient()
			      class Foo { }
			      ```
			       * @param Time for a new tag (pass: valid names)
			       */
			      export function transient<T>(target?: T): T {
			        // ...
			      }
			", None, None),
("
			        /** @jsx h */
			        /** @jsxFrag Fragment */
			        /** @jsxImportSource preact */
			        /** @jsxRuntime automatic (pass: valid jsx names)*/
			      ", Some(serde_json::json!([
        {
          "jsxTags": true,
        },
      ])), None),
("
			      /**
			       * @internal (pass: valid name)
			       */
			      ", None, Some(serde_json::json!({
        "jsdoc": {
        },
      }))),
("
			        /**
			         * @overload
			         * @satisfies (pass: valid names)
			         */
			      ", None, Some(serde_json::json!({
        "jsdoc": {
        },
      }))),
        (
            "
			        /**
			         * @module
			         * A comment related to the module
			         */
			      ",
            None,
            None,
        ),
        // Typed
        ("
      			        /** @default 0 */
      			        let a;
      			      ", Some(serde_json::json!([
        {
          "typed": true,
        },
      ])), None),
("
			        /** @template name */
			        let a;
			      ", Some(serde_json::json!([
        {
          "typed": true,
        },
      ])), None),
("
			        /** @param param - takes information */
			        function takesOne(param) {}
			      ", Some(serde_json::json!([
        {
          "typed": true,
        },
      ])), None),
    ];

    let fail = vec![
        (
            "
        			        /** @typoo {string} (fail: invalid name) */
        			        let a;
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @Param (fail: invalid name)
        			           */
        			          function quux () {
			
        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @foo (fail: invalid name)
        			           */
        			          function quux () {
			
        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @arg foo (fail: invalid name, default aliased)
        			           */
        			          function quux (foo) {
			
        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @param foo (fail: valid name but user preferred)
        			           */
        			          function quux (foo) {

        			          }
        			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "param": "arg",
                },
              },
            })),
        ),
        (
            "
        			          /**
        			           * @constructor foo (fail: invalid name and user preferred)
        			           */
        			          function quux (foo) {

        			          }
        			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "constructor": "cons",
                },
              },
            })),
        ),
        (
            "
        			          /**
                               * @arg foo (fail: invalid name and user preferred)
        			           */
        			          function quux (foo) {

        			          }
        			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "arg": "somethingDifferent",
                },
              },
            })),
        ),
        (
            "
        			          /**
        			           * @param foo (fail: valid name but user preferred)
        			           */
        			          function quux (foo) {

        			          }
        			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "param": "parameter",
                },
              },
            })),
        ),
        (
            "
        			          /**
        			           * @bar foo (fail: invalid name)
        			           */
        			          function quux (foo) {

        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @baz @bar foo (fail: invalid name)
        			           */
        			          function quux (foo) {

        			          }
        			      ",
            Some(serde_json::json!([
              {
                "definedTags": [
                  "bar",
                ],
              },
            ])),
            None,
        ),
        (
            "
        			            /**
        			             * @bar
        			             * @baz (fail: invalid name)
        			             */
        			            function quux (foo) {

        			            }
        			        ",
            Some(serde_json::json!([
              {
                "definedTags": [
                  "bar",
                ],
              },
            ])),
            None,
        ),
        (
            "
        			          /**
        			           * @todo (fail: valid name but blocked)
        			           */
        			          function quux () {

        			          }
        			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "todo": false,
                },
              },
            })),
        ),
        (
            "
        			          /**
        			           * @todo (fail: valid name but blocked)
        			           */
        			          function quux () {

        			          }
        			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "todo": {
                    "message": "Please resolve to-dos or add to the tracker",
                  },
                },
              },
            })),
        ),
        (
            "
        			          /**
        			           * @todo (fail: valid name but blocked)
        			           */
        			          function quux () {

        			          }
        			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "todo": {
                    "message": "Please use x-todo instead of todo",
                    "replacement": "x-todo",
                  },
                },
              },
            })),
        ),
        (
            "
        			          /**
        			           * @property {object} a
        			           * @prop {boolean} b (fail: invalid name, default aliased)
        			           */
        			          function quux () {

        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @abc foo (fail: invalid name and user preferred)
        			           * @abcd bar
        			           */
        			          function quux () {

        			          }
        			      ",
            Some(serde_json::json!([
              {
                "definedTags": [
                  "abcd",
                ],
              },
            ])),
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "abc": "abcd",
                },
              },
            })),
        ),
        (
            "
        			          /**
                               * @abc (fail: invalid name and user preferred)
        			           * @abcd
        			           */
        			          function quux () {

        			          }
        			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "abc": "abcd",
                },
              },
            })),
        ),
        (
            "
        			        /** @jsx h */
        			        /** @jsxFrag Fragment */
        			        /** @jsxImportSource preact */
        			        /** @jsxRuntime automatic */
        			      ",
            None,
            None,
        ),
        (
            "
        			      /**
        			       * @constructor (fail: invalid name)
        			       */
        			      function Test() {
        			        this.works = false;
        			      }
        			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "returns": "return",
                },
              },
            })),
        ),
        (
            "
        			          /**
        			           * @todo (fail: valid name but blocked)
        			           */
        			          function quux () {

        			          }
        			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "todo": {
                    "message": "Please don't use todo",
                  },
                },
              },
            })),
        ),
        // Typed
        (
            "
			        /**
			         * @module
			         * A comment related to the module
			         */
			      ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "/** @type {string} */let a;
        			      ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
        			        /**
        			         * Existing comment.
        			         *  @type {string}
        			         */
        			        let a;
        			      ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
        			      /** @typedef {Object} MyObject
        			       * @property {string} id - my id
        			       */
        			      ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
        			      /**
        			       * @property {string} id - my id
        			       */
        			      ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
        			      /** @typedef {Object} MyObject */
        			      ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
        			      /** @typedef {Object} MyObject
        			       */
        			      ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
        			        /** @abstract */
        			        let a;
        			      ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
        			        const a = {
        			          /** @abstract */
        			          b: true,
        			        };
        			      ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
        			        /** @template */
        			        let a;
        			      ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
        			        /**
        			         * Prior description.
        			         *
        			         * @template
        			         */
        			        let a;
        			      ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
    ];

    let dts_pass = vec![
        (
            "
        			        /** @default 0 */
        			        declare let a;
        			      ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
        			        /** @abstract */
        			        let a;
        			      ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
        			        /** @abstract */
        			        declare let a;
        			      ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
        			        /** @abstract */
        			        { declare let a; }
        			      ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
        			        function test() {
        			          /** @abstract */
        			          declare let a;
        			        }
        			      ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
    ];
    let dts_fail = vec![(
        "
        			        /** @typoo {string} (fail: invalid name) */
        			        let a;
        			      ",
        None,
        None,
    )];

    // Currently only 1 snapshot can be saved under a rule name
    Tester::new(CheckTagNames::NAME, dts_pass, dts_fail).change_rule_path("test.d.ts").test();
    Tester::new(CheckTagNames::NAME, pass, fail).test_and_snapshot();
}
