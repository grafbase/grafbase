<script lang="ts">
	import type { PageData, ActionData } from './$houdini';
	import { enhance } from '$app/forms';

	export let data: PageData;

	export let form: ActionData;

	$: ({ GetAllMessages } = data);
</script>

<h1>Grafbook</h1>

<form method="POST" action="?/auth" use:enhance>
	<button name="role" value="">Set role to public</button>
	<button name="role" value="moderator"> Set role to moderator </button>
	<button name="role" value="admin">Set role to admin</button>
</form>

<form method="POST" action="?/add" use:enhance>
	<fieldset>
		<legend>New message</legend>
		<div>
			<input type="text" name="author" placeholder="Name" required />
		</div>
		<div>
			<textarea name="message" placeholder="Write a message..." required rows={5} />
		</div>
		<div>
			<button type="submit">Add message</button>
		</div>
	</fieldset>
</form>

{#if $GetAllMessages.fetching}
	loading...
{:else if $GetAllMessages.errors?.length}
	{JSON.stringify($GetAllMessages.errors)}
{:else}
	<ul>
		{#each $GetAllMessages.data?.messageCollection?.edges ?? [] as edge (edge?.cursor)}
			<li>
				<p>
					<strong>
						<span>
							{edge?.node.author}
						</span>
						<br />
						<small
							>{new Intl.DateTimeFormat('en-GB', {
								dateStyle: 'medium',
								timeStyle: 'short'
							}).format(edge?.node?.createdAt)}</small
						>
					</strong>
				</p>
				<p>{edge?.node.message}</p>
			</li>
		{/each}
	</ul>
{/if}
