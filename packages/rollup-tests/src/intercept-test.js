const fs = require('fs')
const path = require('path')

/**
 * @param {Mocha.Test | undefined} test
 * @returns {string}
 */
function calcTestId(test) {
  if (!test) {
    throw new Error('test is undefined')
  }
  const paths = test.titlePath()
  return paths.join('@')
}

const failedTestsPath = path.join(__dirname, 'failed-tests.json')

/**
 * @returns {string[]}
 */
function loadFailedTests() {
  if (fs.existsSync(failedTestsPath)) {
    const failedTests = JSON.parse(fs.readFileSync(failedTestsPath, 'utf-8'))
    return failedTests
  }
  return []
}

/**
 * @param {Set<string>} failuresInThisRound
 * @param {Set<string>} failuresInPreviousRounds
 */
function updateFailedTestsJson(failuresInThisRound, failuresInPreviousRounds) {
  const sorted = [...failuresInThisRound, ...failuresInPreviousRounds].sort()
  const formatted = JSON.stringify(sorted, null, 2)
  fs.writeFileSync(path.join(__dirname, 'failed-tests.json'), formatted)
}

/**
 * @type {Set<string>}
 */
const failuresInThisRound = new Set()

const failuresInPreviousRounds = new Set(loadFailedTests())

const ignoreTests = new Set(require('./ignored-tests').ignoreTests)

const status = {
  total: 0,
  failed: 0,
  skipFailed: 0,
  ignored: 0,
  skipped: 0,
  passed: 0,
}

const isUpdateTest = process.env.UPDATE_FAILED === '1'

beforeEach(function skipAlreadyFiledTests() {
  status.total += 1
  const id = calcTestId(this.currentTest)

  if (!isUpdateTest && failuresInPreviousRounds.has(id)) {
    status.skipFailed += 1
    this.currentTest?.skip()
  }

  if (ignoreTests.has(id)) {
    status.ignored += 1
    this.currentTest?.skip()
  }
  const currentTest = this.currentTest
  setTimeout(() => {
    if (currentTest?.state !== 'passed' && currentTest?.state !== 'failed') {
      // Emit a custom error to make it easier to find the test that timed out.
      currentTest?.callback?.(new Error(`Test timed out: [${id}]`))
    }
  }, 500)
})

afterEach(function updateStatus() {
  const id = calcTestId(this.currentTest)
  const state = this.currentTest?.state
  if (state === 'failed') {
    if (!failuresInPreviousRounds.has(id)) {
      failuresInThisRound.add(id)
    }
    status.failed += 1
  } else if (state === 'passed') {
    status.passed += 1
    if (failuresInPreviousRounds.has(id)) {
      failuresInPreviousRounds.delete(id)
    }
  }
})

after(function () {
  const sorted = [...failuresInThisRound].sort()
  const formatted = JSON.stringify(sorted, null, 2)
  console.log('failures', formatted)
  console.table(status)
  if (isUpdateTest) {
    if (failuresInThisRound.size > 0) {
      console.error(
        "Regression detected. Those failed tests won't be added to failed-tests.json.",
      )
    }
    updateFailedTestsJson(new Set(), failuresInThisRound)
  } else {
    if (failuresInThisRound.size > 0) {
      console.log('Regression detected')
      console.log([...failuresInThisRound].sort())
    }
  }
})
