<script lang="ts">
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManHeader from "$lib/components/YakManHeader.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";

	let count = 0;
	let message = "The count is 0";


</script>

<div>

	<YakManButton on:click={() => {
		count -= 1;
		message = "The count is " + count;
	}}>
		Minus
	</YakManButton>

	<YakManButton variant="primary" on:click={() => {
		count += 1;
		message = "The count is " + count;
	}}>
		Plus
	</YakManButton>

	<YakManInput 
		label="Ross"
		placeholder={"123"} 
		bind:value={message} 
	/>
	<p>
		{count}
		{message}
	</p>

</div>

<style>

</style>
