(function(g,f){typeof exports==='object'&&typeof module!=='undefined'?f(exports,require('external')):typeof define==='function'&&define.amd?define(['exports','external'],f):(g=typeof globalThis!=='undefined'?globalThis:g||self,(()=>{const current=g.bundle;const e=g.bundle={};f(e,g.foo$1);e.noConflict=()=>{g.bundle=current;return e};})());})(this,(function(exports,foo$1){'use strict';function _interopNamespaceCompat(e){if(e&&typeof e==='object'&&'default'in e)return e;const n=Object.create(null);if(e){for(const k in e){if(k!=='default'){const d=Object.getOwnPropertyDescriptor(e,k);Object.defineProperty(n,k,d.get?d:{enumerable:true,get:()=>e[k]});}}}n.default=e;return Object.freeze(n)}const foo__namespace=/*#__PURE__*/_interopNamespaceCompat(foo$1);const _missingExportShim=void 0;const foo = 'bar';const other=/*#__PURE__*/Object.freeze({__proto__:null,foo:foo,missing:_missingExportShim});const synthetic = { bar: 'baz' };console.log(foo__namespace.default, foo__namespace, other, bar, _missingExportShim);
const main = 42;exports.default=main;exports.syntheticMissing=synthetic.syntheticMissing;for(const k in foo$1){if(k!=='default'&&!exports.hasOwnProperty(k))Object.defineProperty(exports,k,{enumerable:true,get:()=>foo$1[k]})}Object.defineProperty(exports,'__esModule',{value:true});}));