<script lang="ts">
	import type { Todo } from '$graphql/schema';
	import { getContextClient, mutationStore } from '@urql/svelte';
	import { TodoDeleteDocument, TodoUpdateDocument } from '$graphql/schema';

	export let todo: Todo;

	let input: HTMLInputElement;
	let client = getContextClient();

	let title = todo.title;
	let complete = todo.complete;
	let timer;

	export const debounce = (v) => {
		clearTimeout(timer);
		timer = setTimeout(() => {
			title = v;
		}, 750);
	};

	function updateTodo(t: string, c: boolean) {
		mutationStore({
			client,
			query: TodoUpdateDocument,
			variables: { id: todo.id, title: t, complete: c }
		});
	}

	$: {
		if (title !== todo.title) {
			updateTodo(title, complete);
		}
	}

	function deleteTodo() {
		mutationStore({
			client,
			query: TodoDeleteDocument,
			variables: { id: todo.id }
		});
	}
</script>

<div
	class={`relative rounded-md border p-3 overflow-hidden ${
		todo.complete
			? 'bg-emerald-800 border-emerald-600'
			: 'bg-zinc-50 dark:bg-zinc-800 border-gray-200 dark:border-transparent'
	}`}
>
	{#if todo.complete}
		<div class="absolute text-8xl font-bold left-0 -top-3 text-white text-opacity-5 tracking-wider">
			DONE
		</div>
	{/if}
	<div class="relative">
		<div class="flex justify-between gap-4">
			<div class="flex space-x-1.5 items-center truncate" title={todo.title}>
				<input
					type="checkbox"
					bind:checked={complete}
					on:change={({ target: { value } }) => updateTodo(todo.title, value)}
					class="border-gray-200 text-green-600 dark:border-gray-500 bg-white dark:bg-black rounded accent-green-600 hover:bg-green-600 focus:ring-0"
				/>
				<input
					bind:this={input}
					on:keyup={({ target: { value } }) => debounce(value)}
					value={todo.title}
					class={`bg-transparent focus:outline-0 focus:text-blue-600 focus:dark:text-blue-400 ${
						todo.complete ? 'text-white focus:text-gray-300' : 'text-black dark:text-white'
					}`}
				/>
			</div>
			<button class="text-gray-400 hover:text-red-400 transition" on:click={deleteTodo}>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class={'h-5 w-5 hover:text-red-600 transition' +
						(complete ? 'text-white' : 'text-gray-600')}
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
					stroke-width="2"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
					/>
				</svg>
			</button>
		</div>
		<div class="flex justify-between text-sm mt-2">
			<div
				class={`text-xs px-1 py-0.5 rounded ${
					todo.complete
						? 'bg-green-600 text-white'
						: 'bg-gray-300 dark:bg-gray-600 text-black dark:text-white'
				}`}
			>
				{todo.complete ? 'Completed' : 'Not completed'}
			</div>
		</div>
	</div>
</div>
