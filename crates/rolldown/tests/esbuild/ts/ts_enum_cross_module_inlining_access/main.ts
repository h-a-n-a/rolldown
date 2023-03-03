import { a, b, c, d, e } from './enums'
console.log([
    a.x,
    b['x'],
    c?.x,
    d?.['x'],
    e,
])
