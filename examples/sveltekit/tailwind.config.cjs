/** @type {import('tailwindcss').Config} */
module.exports = {
	darkMode: 'class',
	content: ['./src/**/*.{html,js,svelte,ts,svg}'],
	theme: {
		extend: {}
	},
	plugins: [require('@tailwindcss/forms')]
};
