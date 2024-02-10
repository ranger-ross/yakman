<script lang="ts">
    export let title = "";
    export let hint = "";
    export let text = "";

    let copied = false;

    function copyText() {
        navigator.clipboard.writeText(text);
        copied = true;
    }
</script>

<div class="mt-2">
    <div class="bg-gray-50 text-white p-4 rounded-md">
        <div class="flex justify-between items-center mb-2">
            <span class="text-black font-bold">{title}</span>
            <button
                class="code bg-gray-200 hover:bg-gray-300 text-gray-500 px-3 py-1 rounded-md"
                data-clipboard-target="#code"
                on:click={copyText}
            >
                {#if copied}
                    Copied
                {:else}
                    Copy
                {/if}
            </button>
        </div>
        <div class="overflow-x-auto">
            <code class="text-gray-800 text-sm">
                {text}
            </code>
        </div>
    </div>
    {#if hint}
        <p class="mt-2 px-3 text-gray-600">
            {hint}
        </p>
    {/if}
</div>
