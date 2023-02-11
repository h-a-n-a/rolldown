// @ts-check

const ignoreTests = [
    // The giving code is not valid JavaScript.
    'rollup@function@circular-default-exports: handles circular default exports',
    // Panic: TODO: supports
    'rollup@function@dynamic-import-rewriting: Dynamic import string specifier resolving',
    'rollup@function@deprecated@dynamic-import-name-warn: warns when specifying a custom importer function for formats other than "es"',

    // Import Assertions related
    'rollup@function@import-assertions@plugin-assertions-this-resolve: allows plugins to provide assertions for this.resolve',
]

module.exports = {
    ignoreTests,
}