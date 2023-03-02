package bundler_tests

import (
	"testing"

	"github.com/evanw/esbuild/internal/config"
)

var importstar_suite = suite{
	name: "importstar",
}


func TestExportSelfIIFE(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				export const foo = 123
				export * from './entry'
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			OutputFormat:  config.FormatIIFE,
			AbsOutputFile: "/out.js",
		},
	})
}

func TestExportSelfIIFEWithName(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				export const foo = 123
				export * from './entry'
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			OutputFormat:  config.FormatIIFE,
			AbsOutputFile: "/out.js",
			GlobalName:    []string{"someName"},
		},
	})
}

func TestExportSelfAndRequireSelfCommonJS(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				export const foo = 123
				console.log(require('./entry'))
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			OutputFormat:  config.FormatCommonJS,
			AbsOutputFile: "/out.js",
		},
	})
}

func TestImportExportOtherAsNamespaceCommonJS(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				import * as ns from './foo'
				export {ns}
			`,
			"/foo.js": `
				exports.foo = 123
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			OutputFormat:  config.FormatCommonJS,
			AbsOutputFile: "/out.js",
		},
	})
}

func TestExportOtherCommonJS(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				export {bar} from './foo'
			`,
			"/foo.js": `
				exports.foo = 123
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			OutputFormat:  config.FormatCommonJS,
			AbsOutputFile: "/out.js",
		},
	})
}

func TestEXPORT_OTHER_NESTED_COMMON_JS(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				export {y} from './bar'
			`,
			"/bar.js": `
				export {x as y} from './foo'
			`,
			"/foo.js": `
				exports.foo = 123
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			OutputFormat:  config.FormatCommonJS,
			AbsOutputFile: "/out.js",
		},
	})
}

func TestNamespaceImportMissingCommonJS(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				import * as ns from './foo'
				console.log(ns, ns.foo)
			`,
			"/foo.js": `
				exports.x = 123
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			AbsOutputFile: "/out.js",
		},
	})
}

func TestNamespaceImportUnusedMissingCommonJS(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				import * as ns from './foo'
				console.log(ns.foo)
			`,
			"/foo.js": `
				exports.x = 123
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			AbsOutputFile: "/out.js",
		},
	})
}

func TestNamespaceImportReExportMissingES6(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				import * as ns from './foo'
				console.log(ns, ns.foo)
			`,
			"/foo.js": `
				export {foo} from './bar'
			`,
			"/bar.js": `
				export const x = 123
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			AbsOutputFile: "/out.js",
		},
		expectedCompileLog: `foo.js: ERROR: No matching export in "bar.js" for import "foo"
foo.js: ERROR: No matching export in "bar.js" for import "foo"
`,
	})
}

func TestNamespaceImportReExportUnusedMissingES6(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				import * as ns from './foo'
				console.log(ns.foo)
			`,
			"/foo.js": `
				export {foo} from './bar'
			`,
			"/bar.js": `
				export const x = 123
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			AbsOutputFile: "/out.js",
		},
		expectedCompileLog: `foo.js: ERROR: No matching export in "bar.js" for import "foo"
foo.js: ERROR: No matching export in "bar.js" for import "foo"
`,
	})
}

func TestNamespaceImportReExportStarMissingES6(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				import * as ns from './foo'
				console.log(ns, ns.foo)
			`,
			"/foo.js": `
				export * from './bar'
			`,
			"/bar.js": `
				export const x = 123
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			AbsOutputFile: "/out.js",
		},
		debugLogs: true,
		expectedCompileLog: `entry.js: DEBUG: Import "foo" will always be undefined because there is no matching export in "foo.js"
`,
	})
}

func TestNamespaceImportReExportStarUnusedMissingES6(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				import * as ns from './foo'
				console.log(ns.foo)
			`,
			"/foo.js": `
				export * from './bar'
			`,
			"/bar.js": `
				export const x = 123
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			AbsOutputFile: "/out.js",
		},
		debugLogs: true,
		expectedCompileLog: `entry.js: DEBUG: Import "foo" will always be undefined because there is no matching export in "foo.js"
`,
	})
}

func TestExportStarDefaultExportCommonJS(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				export * from './foo'
			`,
			"/foo.js": `
				export default 'default' // This should not be picked up
				export let foo = 'foo'
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			OutputFormat:  config.FormatCommonJS,
			AbsOutputFile: "/out.js",
		},
	})
}

func TestIssue176(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				import * as things from './folders'
				console.log(JSON.stringify(things))
			`,
			"/folders/index.js": `
				export * from "./child"
			`,
			"/folders/child/index.js": `
				export { foo } from './foo'
			`,
			"/folders/child/foo.js": `
				export const foo = () => 'hi there'
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			AbsOutputFile: "/out.js",
		},
	})
}

func TestReExportStarExternalIIFE(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				export * from "foo"
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			OutputFormat:  config.FormatIIFE,
			AbsOutputFile: "/out.js",
			GlobalName:    []string{"mod"},
			ExternalSettings: config.ExternalSettings{
				PreResolve: config.ExternalMatchers{Exact: map[string]bool{
					"foo": true,
				}},
			},
		},
	})
}

func TestReExportStarExternalES6(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				export * from "foo"
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			OutputFormat:  config.FormatESModule,
			AbsOutputFile: "/out.js",
			ExternalSettings: config.ExternalSettings{
				PreResolve: config.ExternalMatchers{Exact: map[string]bool{
					"foo": true,
				}},
			},
		},
	})
}

func TestReExportStarExternalCommonJS(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				export * from "foo"
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			OutputFormat:  config.FormatCommonJS,
			AbsOutputFile: "/out.js",
			ExternalSettings: config.ExternalSettings{
				PreResolve: config.ExternalMatchers{Exact: map[string]bool{
					"foo": true,
				}},
			},
		},
	})
}

func TestReExportStarAsExternalIIFE(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				export * as out from "foo"
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			OutputFormat:  config.FormatIIFE,
			AbsOutputFile: "/out.js",
			GlobalName:    []string{"mod"},
			ExternalSettings: config.ExternalSettings{
				PreResolve: config.ExternalMatchers{Exact: map[string]bool{
					"foo": true,
				}},
			},
		},
	})
}

// func Testimport_default_namespace_combo_issue446(t *testing.T) {
// 	importstar_suite.expectBundled(t, bundled{
// 		files: map[string]string{
// 			"/external-default2.js": `
// 				import def, {default as default2} from 'external'
// 				console.log(def, default2)
// 			`,
// 			"/external-ns.js": `
// 				import def, * as ns from 'external'
// 				console.log(def, ns)
// 			`,
// 			"/external-ns-default.js": `
// 				import def, * as ns from 'external'
// 				console.log(def, ns, ns.default)
// 			`,
// 			"/external-ns-def.js": `
// 				import def, * as ns from 'external'
// 				console.log(def, ns, ns.def)
// 			`,
// 			"/external-default.js": `
// 				import def, * as ns from 'external'
// 				console.log(def, ns.default)
// 			`,
// 			"/external-def.js": `
// 				import def, * as ns from 'external'
// 				console.log(def, ns.def)
// 			`,
// 			"/internal-default2.js": `
// 				import def, {default as default2} from './internal'
// 				console.log(def, default2)
// 			`,
// 			"/internal-ns.js": `
// 				import def, * as ns from './internal'
// 				console.log(def, ns)
// 			`,
// 			"/internal-ns-default.js": `
// 				import def, * as ns from './internal'
// 				console.log(def, ns, ns.default)
// 			`,
// 			"/internal-ns-def.js": `
// 				import def, * as ns from './internal'
// 				console.log(def, ns, ns.def)
// 			`,
// 			"/internal-default.js": `
// 				import def, * as ns from './internal'
// 				console.log(def, ns.default)
// 			`,
// 			"/internal-def.js": `
// 				import def, * as ns from './internal'
// 				console.log(def, ns.def)
// 			`,
// 			"/internal.js": `
// 				export default 123
// 			`,
// 		},
// 		entryPaths: []string{
// 			"/external-default2.js",
// 			"/external-ns.js",
// 			"/external-ns-default.js",
// 			"/external-ns-def.js",
// 			"/external-default.js",
// 			"/external-def.js",
// 			"/internal-default2.js",
// 			"/internal-ns.js",
// 			"/internal-ns-default.js",
// 			"/internal-ns-def.js",
// 			"/internal-default.js",
// 			"/internal-def.js",
// 		},
// 		options: config.Options{
// 			Mode:         config.ModeBundle,
// 			AbsOutputDir: "/out",
// 			ExternalSettings: config.ExternalSettings{
// 				PreResolve: config.ExternalMatchers{Exact: map[string]bool{
// 					"external": true,
// 				}},
// 			},
// 		},
// 		debugLogs: true,
// 		expectedCompileLog: `internal-def.js: DEBUG: Import "def" will always be undefined because there is no matching export in "internal.js"
// internal-ns-def.js: DEBUG: Import "def" will always be undefined because there is no matching export in "internal.js"
// `,
// 	})
// }

func TestImportDefaultNamespaceComboNoDefault(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry-default-ns-prop.js": `import def, * as ns from './foo'; console.log(def, ns, ns.default)`,
			"/entry-default-ns.js":      `import def, * as ns from './foo'; console.log(def, ns)`,
			"/entry-default-prop.js":    `import def, * as ns from './foo'; console.log(def, ns.default)`,
			"/entry-default.js":         `import def from './foo'; console.log(def)`,
			"/entry-prop.js":            `import * as ns from './foo'; console.log(ns.default)`,
			"/foo.js":                   `export let foo = 123`,
		},
		entryPaths: []string{
			"/entry-default-ns-prop.js",
			"/entry-default-ns.js",
			"/entry-default-prop.js",
			"/entry-default.js",
			"/entry-prop.js",
		},
		options: config.Options{
			Mode:         config.ModeBundle,
			AbsOutputDir: "/out",
		},
		debugLogs: true,
		expectedCompileLog: `entry-default-ns-prop.js: ERROR: No matching export in "foo.js" for import "default"
entry-default-ns-prop.js: DEBUG: Import "default" will always be undefined because there is no matching export in "foo.js"
entry-default-ns.js: ERROR: No matching export in "foo.js" for import "default"
entry-default-prop.js: ERROR: No matching export in "foo.js" for import "default"
entry-default-prop.js: DEBUG: Import "default" will always be undefined because there is no matching export in "foo.js"
entry-default.js: ERROR: No matching export in "foo.js" for import "default"
entry-prop.js: DEBUG: Import "default" will always be undefined because there is no matching export in "foo.js"
`,
	})
}

func TestImportNamespaceUndefinedPropertyEmptyFile(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry-nope.js": `
				import * as js from './empty.js'
				import * as mjs from './empty.mjs'
				import * as cjs from './empty.cjs'
				console.log(
					js.nope,
					mjs.nope,
					cjs.nope,
				)
			`,

			// Note: For CommonJS-style modules, we automatically assign the exports
			// object to the "default" property if there is no property named "default".
			// This is for compatibility with node. So this test intentionally behaves
			// differently from the test above.
			"/entry-default.js": `
				import * as js from './empty.js'
				import * as mjs from './empty.mjs'
				import * as cjs from './empty.cjs'
				console.log(
					js.default,
					mjs.default,
					cjs.default,
				)
			`,

			"/empty.js":  ``,
			"/empty.mjs": ``,
			"/empty.cjs": ``,
		},
		entryPaths: []string{
			"/entry-nope.js",
			"/entry-default.js",
		},
		options: config.Options{
			Mode:         config.ModeBundle,
			AbsOutputDir: "/out",
		},
		debugLogs: true,
		expectedCompileLog: `entry-default.js: DEBUG: Import "default" will always be undefined because there is no matching export in "empty.mjs"
entry-nope.js: WARNING: Import "nope" will always be undefined because the file "empty.js" has no exports
entry-nope.js: WARNING: Import "nope" will always be undefined because the file "empty.mjs" has no exports
entry-nope.js: WARNING: Import "nope" will always be undefined because the file "empty.cjs" has no exports
`,
	})
}

func TestImportNamespaceUndefinedPropertySideEffectFreeFile(t *testing.T) {
	importstar_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry-nope.js": `
				import * as js from './foo/no-side-effects.js'
				import * as mjs from './foo/no-side-effects.mjs'
				import * as cjs from './foo/no-side-effects.cjs'
				console.log(
					js.nope,
					mjs.nope,
					cjs.nope,
				)
			`,

			// Note: For CommonJS-style modules, we automatically assign the exports
			// object to the "default" property if there is no property named "default".
			// This is for compatibility with node. So this test intentionally behaves
			// differently from the test above.
			"/entry-default.js": `
				import * as js from './foo/no-side-effects.js'
				import * as mjs from './foo/no-side-effects.mjs'
				import * as cjs from './foo/no-side-effects.cjs'
				console.log(
					js.default,
					mjs.default,
					cjs.default,
				)
			`,

			"/foo/package.json":        `{ "sideEffects": false }`,
			"/foo/no-side-effects.js":  `console.log('js')`,
			"/foo/no-side-effects.mjs": `console.log('mjs')`,
			"/foo/no-side-effects.cjs": `console.log('cjs')`,
		},
		entryPaths: []string{
			"/entry-nope.js",
			"/entry-default.js",
		},
		options: config.Options{
			Mode:         config.ModeBundle,
			AbsOutputDir: "/out",
		},
		debugLogs: true,
		expectedCompileLog: `entry-default.js: DEBUG: Import "default" will always be undefined because there is no matching export in "foo/no-side-effects.mjs"
entry-nope.js: WARNING: Import "nope" will always be undefined because the file "foo/no-side-effects.js" has no exports
entry-nope.js: WARNING: Import "nope" will always be undefined because the file "foo/no-side-effects.mjs" has no exports
entry-nope.js: WARNING: Import "nope" will always be undefined because the file "foo/no-side-effects.cjs" has no exports
`,
	})
}

