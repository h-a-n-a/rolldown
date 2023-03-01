#!/usr/bin/env node

import { rolldown } from '@rolldown/core'
import { Cli, Command, Option } from 'clipanion'

import { version } from './package.json'

const cli = new Cli({
  binaryName: 'rolldown',
  binaryVersion: version,
})

cli.register(
  class RolldownCli extends Command {
    input = Option.String('--input,-i', {
      required: false,
      description: 'The input file',
    })

    format = Option.String('--format,-f', 'esm', {
      description: 'Type of output (amd, cjs, es, iife, umd, system)',
    })

    async execute() {
      const builder = await rolldown({
        input: this.input,
      })
      const {
        output: [{ code }],
      } = await builder.generate({
        // @ts-expect-error
        format: this.format,
      })
      this.context.stdout.write(code)
    }
  },
)

cli
  .run(process.argv.slice(2), {
    ...Cli.defaultContext,
  })
  .then((status) => {
    process.exit(status)
  })
  .catch((e) => {
    console.error(e)
    process.exit(1)
  })
