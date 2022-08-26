/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: 'class',
  content: ['./src/**/*.{jsx,tsx,js,ts}'],
  theme: {
    extend: {}
  },
  plugins: [require('daisyui')]
}
