module.exports = {
  // see https://medium.com/@jodavid/configure-eslint-with-typescript-prettier-and-vue-js-3-902aae3d1440
  parser: 'vue-eslint-parser',
  parserOptions: {
    parser: '@typescript-eslint/parser',
  },
  extends: [
    // add more generic rulesets here, such as:
    //'eslint:recommended',
    'plugin:vue/vue3-recommended',

    // run prettier as part of eslint: https://github.com/prettier/eslint-plugin-prettier/blob/248cd17f818b5f09a9519576c1e02b9ef26c64d6/README.md?plain=1#L67
    // > Add plugin:prettier/recommended as the last item in the extends array
    // > in your .eslintrc* config file, so that eslint-config-prettier has the
    // > opportunity to override other configs:
    'plugin:prettier/recommended',
  ],
  rules: {
    // override/add rules settings here, such as:
    //'vue/no-unused-vars': 'error'
    eqeqeq: 'error',
  },
}
