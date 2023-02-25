import { defineTestConfig } from '@/utils'
import * as t from 'vitest'

export default defineTestConfig({
  exports(exports) {
    t.expect(exports.file).toBe('foo.js')
  },
})
