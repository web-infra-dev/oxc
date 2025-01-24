// @target:es5
// @module:commonjs
// @experimentaldecorators: true
// from #3108

// @filename: b.ts
import { test } from './a';
function filter(handler) {
    return function (target, propertyKey) {
        // ...
    };
}
class Wat {
    static whatever() {
        // ...
    }
}
babelHelpers.decorate([
    filter(() => test == 'abc')
], Wat, "whatever", null);
