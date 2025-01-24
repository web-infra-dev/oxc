// @target: es5
// @module: commonjs
// @experimentaldecorators: true
// @Filename: 2.ts
import { base } from "./0";
import { foo } from "./0";
/* Comment on the Class Declaration */
let C = class C extends base {
    constructor(prop) {
        super();
    }
};
C = babelHelpers.decorate([
    babelHelpers.decorateParam(0, foo)
], C);
export { C };
