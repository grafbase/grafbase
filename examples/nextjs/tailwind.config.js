/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: 'class',
  content: ['./src/**/*.{jsx,tsx,js,ts}'],
  theme: {
    extend: {
      animation: {
        show: 'show .25s ease-in'
      },
      keyframes: {
        show: {
          from: { opacity: '0' },
          to: { opacity: '1' }
        }
      }
    }
  },
  plugins: [require('@tailwindcss/forms')]
}
