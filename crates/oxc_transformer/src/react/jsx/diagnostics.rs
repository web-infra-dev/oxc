use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

pub fn pragma_and_pragma_frag_cannot_be_set() -> OxcDiagnostic {
    OxcDiagnostic::warning("pragma and pragmaFrag cannot be set when runtime is automatic.")
        .with_help("Remove `pragma` and `pragmaFrag` options.")
}

pub fn import_source_cannot_be_set() -> OxcDiagnostic {
    OxcDiagnostic::warning("importSource cannot be set when runtime is classic.")
        .with_help("Remove `importSource` option.")
}

pub fn namespace_does_not_support(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warning("Namespace tags are not supported by default. React's JSX doesn't support namespace tags. You can set `throwIfNamespace: false` to bypass this warning.")
.with_labels([span0.into()])
}

pub fn valueless_key(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warning("Please provide an explicit key value. Using \"key\" as a shorthand for \"key={true}\" is not allowed.")
.with_labels([span0.into()])
}

pub fn spread_children_are_not_supported(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warning("Spread children are not supported in React.")
        .with_labels([span0.into()])
}
