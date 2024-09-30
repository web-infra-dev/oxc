/// Basically compilerSettings in tsconfig.json
///
/// ## References
/// - [TSConfig Reference](https://www.typescriptlang.org/tsconfig/)
#[derive(Debug)]
pub struct CheckerSettings {
    /// In some cases where no type annotations are present, TypeScript will
    /// fall back to a type of `any` for a variable when it cannot infer the
    /// type.
    ///
    /// Turning on `noImplicitAny` however TypeScript will issue an error
    /// whenever it would have inferred `any`
    ///
    /// <https://www.typescriptlang.org/tsconfig/#noImplicitAny>
    pub no_implicit_any: bool,

    /// With `exactOptionalPropertyTypes` enabled, TypeScript applies stricter
    /// rules around how it handles properties on `type` or `interfaces` which have
    /// a `?` prefix.
    ///
    /// <https://www.typescriptlang.org/tsconfig/#exactOptionalPropertyTypes>
    pub exact_optional_property_types: bool,

    /// When `strictNullChecks` is `false`, `null` and `undefined` are
    /// effectively ignored by the language. This can lead to unexpected errors
    /// at runtime.
    ///
    /// When `strictNullChecks` is `true`, `null` and `undefined` have their own
    /// distinct types and you’ll get a type error if you try to use them where
    /// a concrete value is expected.
    ///
    /// <https://www.typescriptlang.org/tsconfig/#strictNullChecks>
    ///
    /// NOTE: disabling strict null checks is not currently supported. Checking
    /// will panic if this is set to `false`.
    pub(crate) strict_null_checks: bool,
}

impl Default for CheckerSettings {
    fn default() -> Self {
        Self {
            no_implicit_any: false,
            exact_optional_property_types: false,
            strict_null_checks: true,
        }
    }
}
