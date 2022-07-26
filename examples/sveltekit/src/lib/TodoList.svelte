<script lang="ts">
	import getColor from "$lib/utils/get-color";

	import type {TodoList} from "$graphql/schema";

	export let todoList: TodoList;
</script>

<div class="space-y-4 flex-1 min-w-[300px]">
	<div
			class="flex justify-between border-b-2 "
			title={todoList.title}
			style={{ borderColor: getColor(todoList.id) }}
	>
		<h2 class="font-bold text-xl truncate">
			<input
					value={todoList.title}
					class="bg-transparent focus:outline-0 focus:text-blue-600 focus:dark:text-blue-400"
			/>
		</h2>
		<div class="relative z-20">

		</div>
	</div>
	<div class="space-y-4">
		{#if todoList.todos.length}
			{#each todoList.todos as todo}
				hello
			{/each}
		{:else}
			<div>No todos </div>
		{/if}
	</div>
</div>
