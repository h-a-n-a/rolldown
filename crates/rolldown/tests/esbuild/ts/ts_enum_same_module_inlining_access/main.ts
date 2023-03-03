enum a { x = 123 }
enum b { x = 123 }
enum c { x = 123 }
enum d { x = 123 }
enum e { x = 123 }
console.log([
    a.x,
    b['x'],
    c?.x,
    d?.['x'],
    e,
])
