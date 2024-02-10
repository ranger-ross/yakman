<script lang="ts">
    export let title: string;
    export let contentType: string;
    export let data: string;
</script>

<h2 class="text-xl font-bold mb-4">{title}</h2>
<div>
    <p class="text-gray-600 font-semibold mb-1">Content Type</p>

    <code
        class="block bg-gray-100 rounded-md p-2 text-gray-800 whitespace-pre-wrap"
    >
        {contentType}
    </code>
</div>
<div class="mt-4">
    <p class="text-gray-600 font-semibold mb-1">Text</p>
    <code
        class="block bg-gray-100 rounded-md p-2 text-gray-800 whitespace-pre-wrap h-52 overflow-y-auto"
    >
        {data}
    </code>
</div>
