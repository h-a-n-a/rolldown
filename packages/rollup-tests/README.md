The folder `rollup-tests/test` is copied from https://github.com/rollup/rollup/tree/master/test

# Keep the tests up to date

1. Copy https://github.com/rollup/rollup/tree/master/test to replace folder `rollup-tests/test`.

2. Run `yarn test` to check if all tests pass.

# Scripts

## yarn test

Run all tests but skip the ones in `src/failed-tests.json`.

## yarn test:ci

Same as `yarn test` but exit early if any test fails.

## yarn test:update-failed

Only run tests in `src/failed-tests.json`. If the test passes, remove it from the file.
