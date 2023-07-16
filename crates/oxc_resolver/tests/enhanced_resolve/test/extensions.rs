//! <https://github.com/webpack/enhanced-resolve/blob/main/test/extensions.test.js>

use std::path::PathBuf;

use oxc_resolver::{ResolveError, ResolveOptions, Resolver};

fn fixture() -> PathBuf {
    super::fixture().join("extensions")
}

#[test]
fn extensions() -> Result<(), ResolveError> {
    let f = fixture();

    let resolver = Resolver::new(ResolveOptions {
        extensions: vec![".ts".into(), ".js".into()],
        ..ResolveOptions::default()
    });

    #[rustfmt::skip]
    let pass = [
        ("should resolve according to order of provided extensions", "./foo", "foo.ts"),
        ("should resolve according to order of provided extensions (dir index)", "./dir", "dir/index.ts"),
        ("should resolve according to main field in module root", ".", "index.js"),
        ("should resolve single file module before directory", "module", "node_modules/module.js"),
        ("should resolve trailing slash directory before single file", "module/", "node_modules/module/index.ts"),
    ];

    for (comment, request, expected_path) in pass {
        let resolution = resolver.resolve(&f, request)?;
        let path = resolution.path().canonicalize().unwrap();
        let expected = f.join(expected_path).canonicalize().unwrap();
        assert_eq!(path, expected, "{comment} {request} {expected_path}");
    }

    #[rustfmt::skip]
    let fail = [
        ("not resolve to file when request has a trailing slash (relative)", "./foo.js/", f.join("foo.js"))
    ];

    for (comment, request, expected_error) in fail {
        let resolution = resolver.resolve(&f, request);
        let error = ResolveError::NotFound(expected_error.into_boxed_path());
        assert_eq!(resolution, Err(error), "{comment} {request} {resolution:?}");
    }

    Ok(())
}

#[test]
#[ignore = "need to match missingDependencies returned from the resolve function"]
fn default_enforce_extension() {
    // should default enforceExtension to true when extensions includes an empty string
    let fixture = fixture();

    let options = ResolveOptions {
        extensions: vec![".ts".into(), String::new(), ".js".into()],
        ..ResolveOptions::default()
    };

    let resolver = Resolver::new(options);
    let _resolved = resolver.resolve(fixture, "./foo");
}

#[test]
#[ignore = "need to match missingDependencies returned from the resolve function"]
fn respect_enforce_extension() {
    // should respect enforceExtension when extensions includes an empty string
    let fixture = fixture();

    let options = ResolveOptions {
        enforce_extension: false,
        extensions: vec![".ts".into(), String::new(), ".js".into()],
        ..ResolveOptions::default()
    };

    let resolver = Resolver::new(options);
    let _resolved = resolver.resolve(fixture, "./foo");
}
