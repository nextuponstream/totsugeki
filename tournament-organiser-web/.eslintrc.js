module.exports = {
  // Attempted to fix linter but got into problem
  //"parser": "@typescript-eslint/parser",
/*     ,
    "plugins": [
      "@typescript-eslint"
    ], */
    extends: [
      // add more generic rulesets here, such as:
      //'eslint:recommended',
      'plugin:vue/vue3-recommended',
    ],
    rules: {
      // override/add rules settings here, such as:
      //'vue/no-unused-vars': 'error'
    }
  }