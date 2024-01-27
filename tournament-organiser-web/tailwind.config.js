/** @type {import('tailwindcss').Config} */

import { gridSetup, rowSetup, safelist } from './test'
gridSetup['fixed'] = '200px'
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx,vue}",
  ],
  theme: {
    fontFamily: {
      'roboto': ['Roboto', 'sans-serif'],
    },
    extend: {
      gridRow: {
        'span-16384': 'span 16384 / span 16384',
      },
      // FIXME should not be 10k but closer to 1k
      gridTemplateColumns: gridSetup,
      gridRowStart: rowSetup,
    },
  },
  safelist: [
    ...safelist
  ],
}
