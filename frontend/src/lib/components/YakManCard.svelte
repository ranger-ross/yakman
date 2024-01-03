<script lang="ts">
    export let extraClasses = "";
</script>

<div class={`bg-white w-full p-4 rounded shadow-sm ${extraClasses}`}>
    <slot />
</div>
