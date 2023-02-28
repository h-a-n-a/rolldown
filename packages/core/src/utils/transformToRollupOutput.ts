import type { AsyncReturnType } from 'type-fest'
import { Bundler, OutputChunk } from '@rolldown/node-binding'
import type {
  RollupOutput,
  OutputChunk as RollupOutputChunk,
} from '../rollup-types'
import { unimplemented } from '.'

function transformToRollupOutputChunk(chunk: OutputChunk): RollupOutputChunk {
  return {
    type: 'chunk',
    code: chunk.code,
    fileName: chunk.fileName,
    get dynamicImports() {
      throw unimplemented()
      return unimplemented()
    },
    get implicitlyLoadedBefore() {
      throw unimplemented()
      return unimplemented()
    },
    get importedBindings() {
      throw unimplemented()
      return unimplemented()
    },
    get imports() {
      throw unimplemented()
      return unimplemented()
    },
    get modules() {
      throw unimplemented()
      return unimplemented()
    },
    get referencedFiles() {
      throw unimplemented()
      return unimplemented()
    },
    get map() {
      throw unimplemented()
      return unimplemented()
    },
    get exports() {
      throw unimplemented()
      return unimplemented()
    },
    get facadeModuleId() {
      throw unimplemented()
      return unimplemented()
    },
    get isDynamicEntry() {
      throw unimplemented()
      return unimplemented()
    },
    get isEntry() {
      throw unimplemented()
      return unimplemented()
    },
    get isImplicitEntry() {
      throw unimplemented()
      return unimplemented()
    },
    get moduleIds() {
      throw unimplemented()
      return unimplemented()
    },
    get name() {
      throw unimplemented()
      return unimplemented()
    },
  }
}

export function transformToRollupOutput(
  output: AsyncReturnType<Bundler['write']>,
): RollupOutput {
  const [first, ...rest] = output
  return {
    output: [
      transformToRollupOutputChunk(first),
      ...rest.map(transformToRollupOutputChunk),
    ],
  }
}
