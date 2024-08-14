/// Basically compilerSettings in tsconfig.json
#[derive(Debug)]
pub struct CheckerSettings {
    pub no_implicit_any: bool,
    pub exact_optional_property_types: bool,
}

impl Default for CheckerSettings {
    fn default() -> Self {
        Self { no_implicit_any: false, exact_optional_property_types: false }
    }
}
