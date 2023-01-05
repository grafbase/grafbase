<script>
	export let data;

	$: ({ GetAllMessages } = data);
</script>

<pre>{JSON.stringify($GetAllMessages, null, 2)}</pre>

<ul>
	<!-- {#each data.posts as post}
      <li>
        <a href="/blog/{post.slug}">
          {post.title}
        </a>
      </li>
    {/each} -->
</ul>
