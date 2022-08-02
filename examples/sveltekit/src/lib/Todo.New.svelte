<script lang="ts">
	import {getContextClient, mutationStore} from "@urql/svelte";
	import {TodoCreateDocument} from "$graphql/schema";

	export let todoListId : string;

	let title = ''
	let client = getContextClient()

	async function submit(){
		mutationStore({
			client,
			query: TodoCreateDocument,
			variables : { todoListId, title }
		})

		title = ''
	}
</script>

<form on:submit|preventDefault={submit}
		class="flex items-center space-x-2 rounded-lg border-2 border-dashed border-gray-200 dark:border-gray-800 p-3"
>
	<input
			bind:value={title}
			required
			placeholder="Todo title"
			class="w-[177px] bg-gray-50 dark:bg-zinc-800 px-2 py-1 text-sm border border-gray-300 dark:border-gray-800 text-gray-900 text-sm rounded-md focus:ring-blue-500 focus:border-blue-500 block w-full placeholder-gray-400 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
	/>
	<button
			class="bg-blue-800 text-sm rounded-md px-2 py-1 text-white whitespace-nowrap disabled:bg-blue-400"
	>
		Add Todo
	</button>
</form>
