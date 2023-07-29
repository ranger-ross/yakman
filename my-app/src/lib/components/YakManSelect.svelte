<script lang="ts">
    import ArrowDownIcon from "$lib/icons/ArrowDownIcon.svelte";

    export let label: string = "";
    export let value: string | undefined = undefined;
</script>

<div class="w-64">
    <label class="block text-gray-700 text-sm font-bold mb-2">{label}</label>
    <div class="relative">
        <select
            name={label}
            on:change
            bind:value
            class="block appearance-none w-full bg-white border border-gray-400 hover:border-indigo-500 px-4 py-2 pr-8 rounded shadow leading-tight focus:outline-none focus:shadow-outline transition-all duration-200"
        >
            <slot />
        </select>
        <div
            class="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-gray-700"
        >
            <ArrowDownIcon />
        </div>
    </div>
</div>
