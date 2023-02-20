import { a as renamed_a1 } from './a1'


export const a = 0;
const b = { foo: a, a, }
console.log(renamed_a1, a, b.foo)

{
  const a = 1;
  {
    const a = 1;
    {
      const a = 1;
    }
  }
}