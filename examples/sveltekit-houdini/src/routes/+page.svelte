<script lang="ts">
	import type { PageData } from './$houdini';

	export let data: PageData;

	$: ({ ExampleQuery } = data);
</script>

{#if $ExampleQuery.fetching}
	loading...
{:else if $ExampleQuery.errors?.length}
	{JSON.stringify($ExampleQuery)}
{:else}
	{JSON.stringify($ExampleQuery.data)}
{/if}
