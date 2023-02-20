import { defineTestConfig } from '@/utils'
import * as t from 'vitest'

export default defineTestConfig({
	options: {
		plugins: [
			{
				name: '1',
				async transform(code) {
					return code.replace(/MAGIC_NUMBER/g, '3');
				}
			},
			{
				name: '2',
				transform(code) {
					return code.replace(/\d+/g, match => (2 * +match).toString());
				}
			}
		]
	},
	exports(exports) {
		t.expect(exports.magicNumber).toBe(6)
	}
})
