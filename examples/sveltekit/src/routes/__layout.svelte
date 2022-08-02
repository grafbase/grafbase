<script lang="ts">
	import Header from '$lib/Header.svelte';
	import '../app.css';
</script>

<Header />

<main class="relative h-[calc(100vh_-_64px)] flex pl-6 pt-6 pr-12 overflow-x-auto dark:bg-zinc-900">
	<div
		class="fixed left-0 w-6 h-full bg-gradient-to-l from-transparent to-white dark:to-zinc-900"
	/>
	<div
		class="fixed right-0 w-12 h-full bg-gradient-to-r from-transparent to-white dark:to-zinc-900"
	/>
	<slot />
</main>
