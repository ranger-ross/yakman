<script lang="ts">
    import ArrowRightIcon from "$lib/icons/ArrowRightIcon.svelte";

    export let href: string = "";
</script>

<a
    class="text-indigo-600 hover:text-indigo-800 flex items-center text-lg transition-colors duration-200"
    {href}
>
    <slot />
    <ArrowRightIcon />
</a>
