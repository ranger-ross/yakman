<script lang="ts">
    export let variant: "primary" | "secondary" | "danger" = "primary";
    export let type: "button" | "submit" = "button";
    export let disabled: boolean = false;
</script>

{#if variant == "primary"}
    <button
        on:click
        class="bg-indigo-600 hover:bg-indigo-700 disabled:bg-gray-300 text-white text-lg font-bold rounded-md shadow-md py-1 px-4 m-1 transition-colors duration-200"
        {type}
        {disabled}
    >
        <slot />
    </button>
{:else if variant == "secondary"}
    <button
        on:click
        class="bg-white hover:bg-gray-50 disabled:bg-gray-300 rounded-md shadow-sm py-1 px-4 m-1 text-lg text-gray-700 border border-gray-300 transition-colors duration-200"
        {type}
        {disabled}
    >
        <slot />
    </button>
{:else}
    <button
        on:click
        class="bg-red-600 hover:bg-red-700 disabled:bg-gray-300 rounded-md shadow-md py-1 px-4 m-1 text-lg text-white font-bold transition-colors duration-200"
        {type}
        {disabled}
    >
        <slot />
    </button>
{/if}
