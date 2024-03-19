pub fn trim_multiline_comment(s: &str) -> String {
    s.trim()
        .split('\n')
        .map(|line| line.trim().trim_start_matches('*').trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

// For now, just returns inside of most outer braces
pub fn extract_type_range(s: &str) -> Option<(usize, usize)> {
    let mut start = None;
    let mut brace_count = 0;
    for (idx, ch) in s.char_indices() {
        match ch {
            '{' => {
                brace_count += 1;

                if start.is_none() {
                    start = Some(idx + 1);
                }
            }
            '}' => {
                brace_count -= 1;

                if brace_count == 0 {
                    if let Some(start) = start {
                        return Some((start, idx));
                    }
                }
            }
            _ => {}
        }
    }
    None
}

pub fn extract_name_range(s: &str) -> Option<(usize, usize)> {
    let mut start = None;
    for (idx, ch) in s.char_indices() {
        match ch {
            ' ' | '\n' => {
                if let Some(start) = start {
                    return Some((start, idx));
                }
            }
            _ => {
                if start.is_none() {
                    start = Some(idx);
                }
            }
        }
    }

    if let Some(start) = start {
        return Some((start, s.len()));
    }

    None
}

#[cfg(test)]
mod test {
    use super::{extract_name_range, extract_type_range, trim_multiline_comment};

    #[test]
    fn trim_multiline_jsdoc_comments() {
        for (actual, expect) in [
            ("hello", "hello"),
            (
                "
  trim
", "trim",
            ),
            (
                "
 * asterisk
",
                "asterisk",
            ),
            (
                "
 * * li
 * * li
",
                "* li\n* li",
            ),
            (
                "
* list
* list
",
                "list\nlist",
            ),
            (
                "
1

2


3
            ",
                "1\n2\n3",
            ),
        ] {
            assert_eq!(trim_multiline_comment(actual), expect);
        }
    }

    #[test]
    fn extract_type_part_range() {
        for (actual, expect) in [
            ("{t1}", Some("t1")),
            ("{t2 }", Some("t2 ")),
            ("{{ t3: string }}", Some("{ t3: string }")),
            ("{t4} name", Some("t4")),
            (" {t5} ", Some("t5")),
            ("{t6 x", None),
            ("t7", None),
        ] {
            assert_eq!(extract_type_range(actual).map(|(s, e)| &actual[s..e]), expect);
        }
    }

    #[test]
    fn extract_name_part_range() {
        for (actual, expect) in [
            ("n1", Some("n1")),
            ("n2 x", Some("n2")),
            (" n3 ", Some("n3")),
            ("n4\ny", Some("n4")),
            ("", None),
            ("名前5", Some("名前5")),
        ] {
            assert_eq!(extract_name_range(actual).map(|(s, e)| &actual[s..e]), expect);
        }
    }
}
