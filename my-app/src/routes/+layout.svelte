<script>
	import YakManHeader from '$lib/components/YakManHeader.svelte';
import './styles.css';
</script>

<div class="app">
	<YakManHeader />

	<main>
		<slot />
	</main>

</div>

<style>

</style>
