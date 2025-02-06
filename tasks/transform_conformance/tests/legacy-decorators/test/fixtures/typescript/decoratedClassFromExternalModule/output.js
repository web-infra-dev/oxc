// @target: es6
// @experimentaldecorators: true
// @Filename: decorated.ts
function decorate(target) { }
let Decorated = class Decorated {
};
Decorated = babelHelpers.decorate([
    decorate
], Decorated);
export default Decorated;
