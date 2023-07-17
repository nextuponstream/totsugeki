module.exports = {
    // see https://medium.com/@jodavid/configure-eslint-with-typescript-prettier-and-vue-js-3-902aae3d1440
    "parser": "vue-eslint-parser",
    "parserOptions": {
        "parser": "@typescript-eslint/parser",
    },
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