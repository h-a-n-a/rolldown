const { loadFailedTests, calcTestId, updateFailedTestsJson, loadIgnoredTests } = require('./utils')

const alreadyFailedTests = new Set(loadFailedTests())

const ignoredTests = loadIgnoredTests()

beforeEach(function skipAlreadyFiledTests() {
  if (!this.currentTest) {
    throw new Error('No current test')
  }
  const id = calcTestId(this.currentTest)
  if (ignoredTests.has(id) || !alreadyFailedTests.has(id)) {
    this.currentTest?.skip()
  }
  // Easy way to find the test id in the logs
  console.log(id)
  // capture the current test reference
  const currentTest = this.currentTest
  setTimeout(() => {
    if (currentTest.state !== 'passed' && currentTest.state !== 'failed') {
      // Emit a custom error to make it easier to find the test that timed out.
      currentTest.callback?.(new Error(`Test timed out: [${id}]`))
    }
  }, 500)
})

/**
 * @type {Set<string>}
 */
const passedTests = new Set()

afterEach(function updateStatus() {
  if (!this.currentTest) {
    throw new Error('No current test')
  }
  const testId = calcTestId(this.currentTest)
  const state = this.currentTest.state
  if (state === 'passed' && alreadyFailedTests.has(testId)) {
    passedTests.add(testId)
  }
})


after(function printStatus() {
  console.log('Passed tests:', passedTests)
  passedTests.forEach((testId) => alreadyFailedTests.delete(testId))
  updateFailedTestsJson(alreadyFailedTests)
})
