import globby from 'globby'
import { rolldown } from '@rolldown/core'
import path from 'path'
import fs from 'fs'
import { TestConfig } from '@/utils'
import * as t from 'vitest'

function runSamples() {
  const matched = globby.sync('./samples/**/_config.ts', {
    cwd: __dirname,
    absolute: true,
  })
  const samples = matched.map((sampleConfigPath) => {
    const id = path
      .relative(process.cwd(), sampleConfigPath)
      .split(path.sep)
      .join('_')
    return {
      configPath: sampleConfigPath,
      id,
    }
  })
  samples.forEach((sample) => {
    t.test(sample.id, async () => {
      try {
        await runSampleTest(sample.configPath)
      } catch (err) {
        console.log(err)
        throw err
      }
    })
  })
}

async function runSampleTest(sampleConfigPath: string) {
  const sampleDir = path.dirname(sampleConfigPath)
  const distDir = path.join(sampleDir, 'dist')
  const { default: testConfig }: { default: TestConfig } = await import(
    sampleConfigPath
  )
  const build = await rolldown({
    input: './main.js',
    cwd: sampleDir,
    ...testConfig.options,
  })
  fs.rmSync(distDir, { recursive: true, force: true })
  await build.write({ dir: distDir, format: 'cjs' })
  const exports = await import(path.join(distDir, 'main.js'))
  await testConfig.exports?.(exports)
}

runSamples()
