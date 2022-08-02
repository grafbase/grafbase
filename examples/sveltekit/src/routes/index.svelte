			<script lang="ts">
	import {getContextClient, queryStore, setContextClient} from '@urql/svelte';
	import {urqlClient} from "$graphql/urql";
	import {TodoListsDocument} from "$graphql/schema";
	import TodoList from "$lib/TodoList.svelte";
	import TodoListNew from "$lib/TodoList.New.svelte";


	setContextClient(urqlClient);

	$: todoLists = queryStore({
		client: getContextClient(),
		query: TodoListsDocument
	});
</script>

<svelte:head>
	<title>SvelteKit - Todo Example - Grafbase</title>
	<meta
			name="description"
			content="Todo Example leveraging the Grafbase platform"
	/>
</svelte:head>

<div class="flex gap-6">

			{#if $todoLists.fetching}
				<p>Loading...</p>
			{:else if $todoLists.error}
				<p>Something went wrong while loading the todos. {$todoLists.error.message}</p>
			{:else}
				{#each $todoLists.data.todoListCollection.edges as todoList}
					<TodoList todoList={todoList?.node} />
				{/each}
				<TodoListNew />
			{/if}
</div>

