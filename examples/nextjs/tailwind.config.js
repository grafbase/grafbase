/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/pages/**/*.{jsx,tsx}", "./src/components/**/*.{jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        grafbase: "#4A9C6D",
      },
      animation: {
        show: "show .25s ease-in",
      },
      keyframes: {
        show: {
          from: { opacity: "0" },
          to: { opacity: "1" },
        },
      },
    },
  },
  plugins: [],
};
