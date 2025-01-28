class Foo {
    prop = 1;
    foo() {
        return 0;
    }
}
babelHelpers.decorate([
    decorator
], Foo.prototype, "foo", null);
