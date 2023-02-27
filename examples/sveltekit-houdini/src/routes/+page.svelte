<script lang="ts">
	import type { PageData, ActionData } from './$houdini';
	import { enhance } from '$app/forms';

	export let data: PageData;

	export let form: ActionData;

	$: ({ GetAllMessages } = data);
</script>

<form method="POST" use:enhance>
	<button name="role" value="">Set role to public</button>
	<button name="role" value="moderator">
		Set role to moderator
	</button>
	<button name="role" value="admin">Set role to admin</button>
</form>

{#if $GetAllMessages.fetching}
	loading...
{:else if $GetAllMessages.errors?.length}
	{JSON.stringify($GetAllMessages)}
{:else}
	{JSON.stringify($GetAllMessages.data)}
{/if}
