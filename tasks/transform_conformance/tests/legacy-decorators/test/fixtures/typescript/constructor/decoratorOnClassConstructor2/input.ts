// @target: es5
// @module: commonjs
// @experimentaldecorators: true

// @Filename: 2.ts
import {base} from "./0.ts"
import {foo} from "./0.ts"
export class C  extends base{
    constructor(@foo prop: any) {
        super();
    }
}
