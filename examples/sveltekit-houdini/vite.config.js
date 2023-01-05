import houdini from 'houdini/vite';
import { sveltekit } from '@sveltejs/kit/vite';

/** @type {import('vite').UserConfig} */
const config = {
	plugins: [houdini(), sveltekit()]
};

export default config;
