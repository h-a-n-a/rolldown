# Test

Install [cargo-insta](https://crates.io/crates/cargo-insta) to update the snapshots easily.

# Scripts

## yarn build

Build `@rolldown/core` include its dependencies.

## yarn type-check

Run `type-check` script in each package.

## yarn test:ci

Run `test:ci` script in each package.

# Profile

With `TRACING=1`, rolldown will emit a `trace-xxx.json` file, which describe the time cost of each part.

With `RUST_LOG={TRACE | DEBUG | INFO | WARN | ERROR}`, the internal logging of rolldown will be enabled.
