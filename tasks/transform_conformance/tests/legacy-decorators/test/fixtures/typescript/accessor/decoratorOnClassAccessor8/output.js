class A {
    get x() { return 0; }
    set x(value) { }
}
babelHelpers.decorate([
    dec
], A.prototype, "x", null);
class B {
    get x() { return 0; }
    set x(value) { }
}
babelHelpers.decorate([
    dec
], B.prototype, "x", null);
class C {
    set x(value) { }
    get x() { return 0; }
}
babelHelpers.decorate([
    dec
], C.prototype, "x", null);
class D {
    set x(value) { }
    get x() { return 0; }
}
babelHelpers.decorate([
    dec
], D.prototype, "x", null);
class E {
    get x() { return 0; }
}
babelHelpers.decorate([
    dec
], E.prototype, "x", null);
class F {
    set x(value) { }
}
babelHelpers.decorate([
    dec
], F.prototype, "x", null);
