let A = class A {
};
A = babelHelpers.decorate([
    dec
], A);
let B = class B {
    constructor(x) { }
};
B = babelHelpers.decorate([
    dec
], B);
let C = class C extends A {
};
C = babelHelpers.decorate([
    dec
], C);
