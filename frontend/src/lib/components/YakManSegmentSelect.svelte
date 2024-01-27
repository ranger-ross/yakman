<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let options: string[] = [];
    export let selectedOption: string = options[0];

    const dispatch = createEventDispatcher<{ select: string }>();

    function onSelect(option: string) {
        selectedOption = option;
        dispatch("select", option);
    }
</script>

<div class="flex gap-2">
    {#each options as option}
        {#if selectedOption === option}
            <div
                class="bg-white rounded-lg py-1.5 px-2 text-gray-900 font-bold cursor-pointer border-gray-400 border-solid border-2 hover:border-gray-500 transition-all duration-200"
            >
                {option}
            </div>
        {:else}
            <!-- svelte-ignore a11y-click-events-have-key-events -->
            <!-- svelte-ignore a11y-no-static-element-interactions -->
            <div
                class="bg-white rounded-lg py-1.5 px-2 text-gray-500 font-bold cursor-pointer border-white border-solid border-2 hover:border-gray-200 transition-all duration-300"
                on:click={() => onSelect(option)}
            >
                {option}
            </div>
        {/if}
    {/each}
</div>
