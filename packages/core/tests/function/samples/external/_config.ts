import { defineTestConfig } from '@/utils'
import { isAbsolute } from 'path'
import * as t from 'vitest'

export default defineTestConfig({
  options: {
    external: (id) => {
      return id == 'path'
    },
  },
  exports(exports) {
    t.expect(exports.default).toBe(isAbsolute)
  },
})
