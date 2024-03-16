<script lang="ts">
    import ContentTypePill from "$lib/components/ContentTypePill.svelte";
    import MonacoEditor from "$lib/components/MonacoEditor.svelte";
    import { contentTypeToMonacoLanguage } from "$lib/utils/content-type-utils";

    export let title: string;
    export let contentType: string;
    export let data: string;

    $: editorLanguage = contentTypeToMonacoLanguage(contentType);
</script>

<h2 class="text-xl font-bold mb-4">{title}</h2>
<div>
    <p class="text-gray-600 font-semibold mb-1">Content Type</p>
    <ContentTypePill {contentType} />
</div>
<div class="mt-4">
    <p class="text-gray-600 font-semibold mb-1">Text</p>

    <div class="h-64">
        <MonacoEditor content={data} language={editorLanguage} disabled={true} />
    </div>
</div>
