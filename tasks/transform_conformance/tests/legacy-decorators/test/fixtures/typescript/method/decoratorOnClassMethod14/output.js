class Foo {
    prop = () => {
        return 0;
    };
    foo() {
        return 0;
    }
}
babelHelpers.decorate([
    decorator
], Foo.prototype, "foo", null);
