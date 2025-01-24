// @target: es5
// @module: commonjs
// @experimentaldecorators: true
// @Filename: 2.ts
import { base } from "./0.ts";
import { foo } from "./0.ts";
let C = class C extends base {
    constructor(prop) {
        super();
    }
};
C = babelHelpers.decorate([
    babelHelpers.decorateParam(0, foo)
], C);
export { C };
