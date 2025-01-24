use quote::format_ident;
use syn::Ident;

/// Reserved word in Rust.
/// From https://doc.rust-lang.org/reference/keywords.html
#[rustfmt::skip]
static RESERVED_NAMES: &[&str] = &[
    // Strict keywords
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for", "if",
    "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self",
    "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while", "async",
    "await", "dyn",
    // Reserved keywords
    "abstract", "become", "box", "do", "final", "macro", "override", "priv", "typeof", "unsized",
    "virtual", "yield", "try",
    // Weak keywords
    "macro_rules", "union", // "dyn" also listed as a weak keyword, but is already on strict list
];

/// Returns `true` if `name` is a reserved word in Rust.
pub fn is_reserved_name(name: &str) -> bool {
    RESERVED_NAMES.contains(&name)
}

/// Create an [`Ident`] from a string.
///
/// If the name is a reserved word, it's prepended with `r#`.
/// e.g. `type` -> `r#type`.
pub fn create_ident(name: &str) -> Ident {
    if is_reserved_name(name) {
        format_ident!("r#{name}")
    } else {
        format_ident!("{name}")
    }
}
