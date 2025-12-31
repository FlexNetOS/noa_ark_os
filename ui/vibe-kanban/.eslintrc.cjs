/**
 * ESLint configuration scoped to the Vibe Kanban Next.js app.
 * Ensures accessibility checks for the AI assist UI components.
 */

module.exports = {
  root: true,
  parser: "@typescript-eslint/parser",
  parserOptions: {
    ecmaVersion: 2022,
    sourceType: "module",
    project: ["./tsconfig.json"],
  },
  plugins: ["@typescript-eslint", "react", "react-hooks", "jsx-a11y"],
  extends: [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:react/recommended",
    "plugin:react-hooks/recommended",
    "plugin:jsx-a11y/recommended",
    "plugin:@typescript-eslint/stylistic",
  ],
  env: {
    browser: true,
    es2022: true,
    node: true,
  },
  settings: {
    react: {
      version: "detect",
    },
  },
  rules: {
    "react/react-in-jsx-scope": "off",
    "@typescript-eslint/explicit-function-return-type": "off",
    "@typescript-eslint/no-unused-vars": ["error", { argsIgnorePattern: "^_" }],
    "@typescript-eslint/consistent-type-definitions": "off",
    "@typescript-eslint/array-type": "off",
    "jsx-a11y/no-static-element-interactions": "off",
    "jsx-a11y/label-has-associated-control": "off",
    "react-hooks/set-state-in-effect": "off",
  },
  ignorePatterns: ["next.config.mjs", "tailwind.config.ts", "postcss.config.js"],
};
