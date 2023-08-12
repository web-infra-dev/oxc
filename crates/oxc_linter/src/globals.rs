//! [Globals](https://github.com/sindresorhus/globals/blob/main/globals.json)
//! Each global is given a value of true or false.
//! A value of true indicates that the variable may be overwritten.
//! A value of false indicates that the variable should be considered read-only.

use phf::{phf_map, Map};

pub const BUILTINS: Map<&'static str, bool> = phf_map! {
    "AggregateError" => false,
    "Array" => false,
    "ArrayBuffer" => false,
    "Atomics" => false,
    "BigInt" => false,
    "BigInt64Array" => false,
    "BigUint64Array" => false,
    "Boolean" => false,
    "constructor" => false,
    "DataView" => false,
    "Date" => false,
    "decodeURI" => false,
    "decodeURIComponent" => false,
    "encodeURI" => false,
    "encodeURIComponent" => false,
    "Error" => false,
    "escape" => false,
    "eval" => false,
    "EvalError" => false,
    "FinalizationRegistry" => false,
    "Float32Array" => false,
    "Float64Array" => false,
    "Function" => false,
    "globalThis" => false,
    "hasOwnProperty" => false,
    "Infinity" => false,
    "Int16Array" => false,
    "Int32Array" => false,
    "Int8Array" => false,
    "isFinite" => false,
    "isNaN" => false,
    "isPrototypeOf" => false,
    "JSON" => false,
    "Map" => false,
    "Math" => false,
    "NaN" => false,
    "Number" => false,
    "Object" => false,
    "parseFloat" => false,
    "parseInt" => false,
    "Promise" => false,
    "propertyIsEnumerable" => false,
    "Proxy" => false,
    "RangeError" => false,
    "ReferenceError" => false,
    "Reflect" => false,
    "RegExp" => false,
    "Set" => false,
    "SharedArrayBuffer" => false,
    "String" => false,
    "Symbol" => false,
    "Diagnostic" => false,
    "toLocaleString" => false,
    "toString" => false,
    "TypeError" => false,
    "Uint16Array" => false,
    "Uint32Array" => false,
    "Uint8Array" => false,
    "Uint8ClampedArray" => false,
    "undefined" => false,
    "unescape" => false,
    "URIError" => false,
    "valueOf" => false,
    "WeakMap" => false,
    "WeakRef" => false,
    "WeakSet" => false
};

pub const PRE_DEFINE_VAR: Map<&'static str, bool> = phf_map!{
    "undefined" => false,
    "null" => false,
    "Infinity" => false,
    "NaN" => false,
    "eval" => false,
    "arguments" => false,
    "this" => false,
    "super" => false,
    "globalThis" => false,
};
