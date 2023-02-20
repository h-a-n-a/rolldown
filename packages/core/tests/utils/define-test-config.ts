import type { InputOptions } from '@rolldown/core'

export interface TestConfig {
    options?: InputOptions,
    exports?(exports: any): void | Promise<void>,
}

export function defineTestConfig(config: TestConfig): TestConfig {
    return config
}