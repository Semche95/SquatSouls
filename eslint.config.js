import js from '@eslint/js';
import vue from 'eslint-plugin-vue';
import globals from 'globals';

export default [
  js.configs.recommended,
  ...vue.configs['flat/recommended'],
  {
    linterOptions: { reportUnusedDisableDirectives: 'error' },
    languageOptions: { globals: globals.browser },
    rules: {
      // Leaving a property out of an object is written by destructuring it next to a rest.
      'no-unused-vars': ['error', { ignoreRestSiblings: true }],
      'array-callback-return': 'error',
      'consistent-return': 'error',
      // `== null` stands for null or undefined.
      eqeqeq: ['error', 'smart'],
      'no-console': 'error',
      'no-else-return': 'error',
      'no-lonely-if': 'error',
      'no-loop-func': 'error',
      'no-param-reassign': 'error',
      'no-promise-executor-return': 'error',
      'no-return-assign': 'error',
      'no-shadow': 'error',
      'no-unneeded-ternary': 'error',
      'no-unused-expressions': 'error',
      // Only what runs before its declaration: a function may use what the module declares after it.
      'no-use-before-define': ['error', { functions: false, variables: false }],
      'no-useless-return': 'error',
      'no-var': 'error',
      'object-shorthand': 'error',
      'prefer-arrow-callback': 'error',
      'prefer-const': 'error',
      'prefer-template': 'error',
      'require-await': 'error',
      'vue/block-order': ['error', { order: ['script', 'template', 'style'] }],
      // Written like the rest of the templates: `<input />`, `<MyComponent />`.
      'vue/html-self-closing': ['error', { html: { void: 'always', normal: 'any', component: 'always' } }],
      'vue/eqeqeq': ['error', 'smart'],
      // `i18n-t` is registered globally by vue-i18n.
      'vue/no-undef-components': ['error', { ignorePatterns: ['i18n-t'] }],
      'vue/no-undef-properties': 'error',
      'vue/no-unused-properties': 'error',
      'vue/no-unused-refs': 'error',
      'vue/no-useless-v-bind': 'error',
      'vue/prefer-true-attribute-shorthand': 'error',
      'vue/require-explicit-emits': 'error',
      // An optional prop left out is undefined, which is what each of them means here.
      'vue/require-default-prop': 'off',
      // Layout only: the templates keep their own.
      'vue/max-attributes-per-line': 'off',
      'vue/singleline-html-element-content-newline': 'off',
    },
  },
  // The mocks of the Tauri API are async like the functions they replace, with nothing to await.
  { files: ['src/tests/**'], rules: { 'require-await': 'off' } },
  { files: ['scripts/**', '*.config.js'], languageOptions: { globals: globals.node } },
];
