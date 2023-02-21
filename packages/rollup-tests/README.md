The folder `rollup-tests/test` is copied from https://github.com/rollup/rollup/tree/master/test

# Keep the tests up to date

1. Copy https://github.com/rollup/rollup/tree/master/test to replace folder `rollup-tests/test`.

2. execute `pnpm run test:update`

## `pnpm run test`

The script will run the tests and skip the tests in `rollup-tests/failed-tests.json`.

## `pnpm run test:update`

The script will run the tests and collected failed tests to `rollup-tests/failed-tests.json`.

If a test is already in `rollup-tests/failed-tests.json`, it will be skipped.
