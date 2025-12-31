import type { TestingLibraryMatchers } from "@testing-library/jest-dom/matchers";

type TestingLibraryMatchersExtension<T = unknown> = TestingLibraryMatchers<T, unknown> & {
  readonly _testingLibraryMatchersBrand?: never;
};

declare module "vitest" {
  interface Assertion<T = unknown> extends TestingLibraryMatchersExtension<T> {
    /**
     * Branded property so eslint does not treat the augmentation as empty.
     * This has no runtime impact because it is typed to `never`.
     */
    readonly __testingLibraryAssertionBrand?: never;
  }

  interface AsymmetricMatchersContaining extends TestingLibraryMatchersExtension<unknown> {
    /** Matches Assertion branding for asymmetric helpers. */
    readonly __testingLibraryAsymmetricBrand?: never;
  }
}
