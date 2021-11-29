# Snapshot report for `packages/node/__test__/core.spec.ts`

The actual snapshot is saved in `core.spec.ts.snap`.

Generated by [AVA](https://avajs.dev).

## should be able to bootstrap

> Snapshot 1

    `function add(a, b) {␊
        return a + b;␊
    }␊
    function mul(a, b) {␊
        let result = 0;␊
        for(let i = 0; i < a; i++){␊
            result = add(result, b);␊
        }␊
        return result;␊
    }␊
    console.log(mul(8, 9));␊
    `