// @target:es5
// @module:commonjs
// @experimentaldecorators: true
// from #3108

// @filename: b.ts
import { test } from './a';

function filter(handler: any) {
    return function (target: any, propertyKey: string) {
        // ...
    };
}

class Wat {
    @filter(() => test == 'abc')
    static whatever() {
        // ...
    }
}
