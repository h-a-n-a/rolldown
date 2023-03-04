import 'zx/globals'

for (const i in Array(10).fill(null)) {
  fs.ensureDir('./temp/threejs10x')
  await $`cp -r ./temp/threejs/src ./temp/threejs10x/copy${i}`
}

const entryCode = Array(10)
  .fill(null)
  .map((_, i) => i)
  .map(
    (i) =>
      `import * as copy${i} from './copy${i}/Three.js'\nexport { copy${i} }`,
  )
  .join('\n')

fs.writeFile('./temp/threejs10x/main.js', entryCode)
