// Tailwind Config
module.exports = {
  safelist: [
    {
      // ugly workaround to having no minify pipeline
      // TODO minify
      pattern: /./, // the "." means "everything"
      variants: ['hover', 'responsive'],
    },
  ],
  theme: {}
}
