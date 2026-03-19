module.exports = {
  ignorePatterns: [
    'dist',
    'node_modules',
    '*.config.js',
    '*.config.ts',
    'coverage',
    'playwright-report',
    'vitest-reports',
    'scripts',
    'tests',
    'e2e',
    'vitest-plugins',
    '**/*.test.ts',
    '**/*.test.tsx',
    'src/test',
    'vitest-reporters',
    '**/test-*.js',
    '**/test_*.js',
    'vitest-slow-test-reporter.ts',
    'src/wasm/pkg',
    'src/types/generated.ts',
  ],
  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaVersion: 2020,
    sourceType: 'module',
    ecmaFeatures: {
      jsx: true,
    },
  },
  extends: ['eslint:recommended', 'plugin:@typescript-eslint/recommended', 'plugin:react-hooks/recommended'],
  plugins: [
    '@typescript-eslint',
    'react-hooks',
    'local-rules',
  ],
  rules: {
    'no-console': ['error', { allow: ['warn', 'error'] }],
    '@typescript-eslint/no-explicit-any': 'error',
    '@typescript-eslint/no-unused-vars': ['error', {
      argsIgnorePattern: '^_',
      varsIgnorePattern: '^_',
      caughtErrorsIgnorePattern: '^_'
    }],
    // Enable custom test naming convention rule as warning
    'local-rules/test-naming-convention': 'warn',
  },
  env: {
    browser: true,
    es2020: true,
    node: true,
  },
  settings: {
    react: {
      version: 'detect',
    },
  },
};
