<script lang="ts">
	import type { PageData } from './$houdini';

	export let data: PageData;

	$: ({ GetAllMessages } = data);
</script>

{#if $GetAllMessages.fetching}
	loading...
{:else if $GetAllMessages.errors?.length}
	{JSON.stringify($GetAllMessages)}
{:else}
	{JSON.stringify($GetAllMessages.data)}
{/if}
