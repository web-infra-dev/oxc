use super::utils;

//
// Structs
//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JSDocTag<'a> {
    raw_body: &'a str,
    pub kind: &'a str,
}

impl<'a> JSDocTag<'a> {
    /// kind: Does not contain the `@` prefix
    /// raw_body: The body part of the tag, after the `@kind {HERE...}`
    pub fn new(kind: &'a str, raw_body: &'a str) -> JSDocTag<'a> {
        Self { raw_body, kind }
    }

    pub fn comment(&self) -> String {
        utils::trim_multiline_comment(self.raw_body)
    }

    pub fn as_param(&self) -> (Option<&str>, Option<String>, Option<String>) {
        let mut breakpoints = vec![];
        let mut in_braces = false;
        // Use indices for string slices
        let mut chars = self.raw_body.char_indices().peekable();

        // Skip leading spaces
        while let Some((_, ch)) = chars.peek() {
            if !(*ch == ' ' || *ch == '\n') {
                break;
            }
            chars.next();
        }

        // Find 2 breakpoints: {type}|name|comment
        // - type may contain line breaks and spaces
        // - comment may contain line breaks
        'outer: while let Some((_, ch)) = chars.peek() {
            if breakpoints.len() == 2 {
                break;
            }

            match ch {
                '{' => in_braces = true,
                '}' => in_braces = false,
                ' ' | '\n' if !in_braces => {
                    for (idx, ch) in chars.by_ref() {
                        if ch != ' ' {
                            breakpoints.push(idx);
                            continue 'outer;
                        }
                    }
                }
                _ => {}
            }

            chars.next();
        }

        match breakpoints.len() {
            // {type} name3 comment
            //
            // name
            // com
            // ment
            2 => {
                let type_or_name = &self.raw_body[..breakpoints[0]].trim();
                if type_or_name.starts_with('{') {
                    let r#type = &type_or_name[1..type_or_name.len() - 1].trim();
                    let name = &self.raw_body[breakpoints[0]..breakpoints[1]].trim();
                    let comment = &self.raw_body[breakpoints[1]..];
                    (
                        Some(*r#type),
                        Some((*name).to_string()),
                        Some(utils::trim_multiline_comment(comment)),
                    )
                } else {
                    let name = type_or_name;
                    let comment = &self.raw_body[breakpoints[0]..].trim();
                    (None, Some((*name).to_string()), Some(utils::trim_multiline_comment(comment)))
                }
            }
            // ```
            // {type} name
            //
            // name comment
            //
            // name
            // comment
            // ```
            1 => {
                let type_or_name = &self.raw_body[..breakpoints[0]].trim();
                if type_or_name.starts_with('{') {
                    let r#type = &type_or_name[1..type_or_name.len() - 1].trim();
                    let name = &self.raw_body[breakpoints[0]..].trim();
                    (Some(*r#type), Some((*name).to_string()), None)
                } else {
                    let name = type_or_name;
                    let comment = &self.raw_body[breakpoints[0]..].trim();
                    (None, Some((*name).to_string()), Some(utils::trim_multiline_comment(comment)))
                }
            }
            // name
            // {type}
            // {type not closed
            _ => {
                let type_or_name = &self.raw_body.trim();
                if type_or_name.starts_with('{') {
                    let r#type = &type_or_name[1..type_or_name.len() - 1].trim();
                    (Some(r#type), None, None)
                } else {
                    let name = type_or_name;
                    (None, Some((*name).to_string()), None)
                }
            }
        }
    }

    // pub fn as_returns(&self) {}
}

#[cfg(test)]
mod test {
    use super::JSDocTag;

    #[test]
    fn parses_comment() {
        assert_eq!(JSDocTag::new("foo1", "").comment(), "");
        assert_eq!(JSDocTag::new("foo2", "bar").comment(), "bar");
        assert_eq!(JSDocTag::new("foo3", " a \n z ").comment(), "a\nz");
        assert_eq!(JSDocTag::new("foo4", "* a\n *  \n z \n\n").comment(), "a\nz");
        assert_eq!(
            JSDocTag::new("foo5", "comment and {@inline tag}!").comment(),
            "comment and {@inline tag}!"
        );
    }

    #[test]
    fn parses_parameter_tag() {
        assert_eq!(
            JSDocTag::new("param", "name1").as_param(),
            (None, Some("name1".to_string()), None)
        );
        assert_eq!(
            JSDocTag::new("arg", "{type2} name2").as_param(),
            (Some("type2"), Some("name2".to_string()), None)
        );
        assert_eq!(
            JSDocTag::new("arg", " {type3 }  name3 ").as_param(),
            (Some("type3"), Some("name3".to_string()), None)
        );
        assert_eq!(
            JSDocTag::new("arg", "{{ x: 1 }} name4").as_param(),
            (Some("{ x: 1 }"), Some("name4".to_string()), None)
        );
        assert_eq!(
            JSDocTag::new("arg", "{type5} name5 comment5").as_param(),
            (Some("type5"), Some("name5".to_string()), Some("comment5".to_string()))
        );
        assert_eq!(
            JSDocTag::new("arg", "{type6} 変数6 あいうえ\nお6").as_param(),
            (Some("type6"), Some("変数6".to_string()), Some("あいうえ\nお6".to_string()))
        );
        assert_eq!(
            JSDocTag::new("arg", "{type7}\nname7").as_param(),
            (Some("type7"), Some("name7".to_string()), None)
        );
        assert_eq!(
            JSDocTag::new("arg", "{type8}\nname8\ncomment8").as_param(),
            (Some("type8"), Some("name8".to_string()), Some("comment8".to_string()))
        );
        assert_eq!(
            JSDocTag::new("arg", "\nname9").as_param(),
            (None, Some("name9".to_string()), None)
        );
        assert_eq!(
            JSDocTag::new("arg", "name10\ncom\nment10").as_param(),
            (None, Some("name10".to_string()), Some("com\nment10".to_string()))
        );
        assert_eq!(JSDocTag::new("arg", "{type11}").as_param(), (Some("type11"), None, None));

        // TODO: More tests!
    }

    //         assert_eq!(
    //             parse_from_full_text("/** @param */").1,
    //             vec![JSDocTag {
    //                 kind: JSDocTagKind::Parameter(Param { name: "", r#type: None }),
    //                 comment: String::new(),
    //             },]
    //         );
    //         assert_eq!(
    //             parse_from_full_text("/** @param @noop */").1,
    //             vec![
    //                 JSDocTag {
    //                     kind: JSDocTagKind::Parameter(Param { name: "", r#type: None }),
    //                     comment: String::new(),
    //                 },
    //                 JSDocTag { kind: JSDocTagKind::Unknown("noop"), comment: String::new() },
    //             ]
    //         );
    //         assert_eq!(
    //             parse_from_full_text("/** @param name */").1,
    //             vec![JSDocTag {
    //                 kind: JSDocTagKind::Parameter(Param { name: "name", r#type: None }),
    //                 comment: String::new(),
    //             },]
    //         );
    //         assert_eq!(
    //             parse_from_full_text("/** @param {str} name */").1,
    //             vec![JSDocTag {
    //                 kind: JSDocTagKind::Parameter(Param {
    //                     name: "name",
    //                     r#type: Some(ParamType { value: "str" })
    //                 }),
    //                 comment: String::new(),
    //             },]
    //         );
    //         assert_eq!(
    //             parse_from_full_text("/** @param {str} name comment */").1,
    //             vec![JSDocTag {
    //                 kind: JSDocTagKind::Parameter(Param {
    //                     name: "name",
    //                     r#type: Some(ParamType { value: "str" })
    //                 }),
    //                 comment: "comment".to_string(),
    //             },]
    //         );
    //         assert_eq!(
    //             parse_from_full_text("/** @param {str} name comment */"),
    //             parse_from_full_text("/** @param {str} name - comment */"),
    //         );
    //         assert_eq!(
    //             parse_from_full_text("/** @param {str} name comment */"),
    //             parse_from_full_text(
    //                 "/** @param {str} name
    //     comment */"
    //             ),
    //         );
    //         assert_eq!(
    //             parse_from_full_text(
    //                 "/** @param {str} name
    //     comment */"
    //             ),
    //             parse_from_full_text(
    //                 "/**
    //                       * @param {str} name
    //                       * comment
    //                       */"
    //             ),
    //         );

    //         assert_eq!(
    //             parse_from_full_text(
    //                 "
    //                     /**
    //                      * @param {boolean} a
    //                      * @param {string b
    //                      * @param {string} c comment
    //                      * @param {Num} d - comment2
    //                      */
    //             "
    //             )
    //             .1,
    //             vec![
    //                 JSDocTag {
    //                     kind: JSDocTagKind::Parameter(Param {
    //                         name: "a",
    //                         r#type: Some(ParamType { value: "boolean" })
    //                     }),
    //                     comment: String::new(),
    //                 },
    //                 JSDocTag {
    //                     kind: JSDocTagKind::Parameter(Param {
    //                         name: "b",
    //                         r#type: Some(ParamType { value: "string" })
    //                     }),
    //                     comment: String::new(),
    //                 },
    //                 JSDocTag {
    //                     kind: JSDocTagKind::Parameter(Param {
    //                         name: "c",
    //                         r#type: Some(ParamType { value: "string" })
    //                     }),
    //                     comment: "comment".to_string(),
    //                 },
    //                 JSDocTag {
    //                     kind: JSDocTagKind::Parameter(Param {
    //                         name: "d",
    //                         r#type: Some(ParamType { value: "Num" })
    //                     }),
    //                     comment: "comment2".to_string(),
    //                 },
    //             ]
    //         );
    //     }
}
