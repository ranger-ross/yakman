<script lang="ts">
    export let variant: "primary" | "secondary" = "primary";
    export let type: "button" | "submit" = "button"
</script>

{#if variant == "primary"}
    <button
        on:click
        class="bg-indigo-600 hover:bg-indigo-700 disabled:bg-gray-300 text-white text-lg font-bold rounded-md shadow-md py-1 px-4 m-1 transition-colors duration-200"
        type={type}
    >
        <slot />
    </button>
{:else}
    <button
        on:click
        class="bg-white hover:bg-gray-50 disabled:bg-gray-300 rounded-md shadow-sm py-1 px-4 m-1 text-lg text-gray-700 border border-gray-300 transition-colors duration-200"
        type={type}
    >
        <slot />
    </button>
{/if}
