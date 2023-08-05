<script lang="ts">
    import { page } from "$app/stores";
    import LabelPill from "$lib/components/LabelPill.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import YakManTextArea from "$lib/components/YakManTextArea.svelte";
    import { trpc } from "$lib/trpc/client";
    import type { YakManInstanceRevision } from "$lib/types/types";
    import { onMount } from "svelte";

    export let currentRevision: string | null = null;
    export let pendingRevision: string | null = null;
    export let sortedRevisions: YakManInstanceRevision[] = [];

    let selectedRevisionData = {
        revision: "",
        data: "",
        contentType: "",
        labels: [] as string[],
    };

    onMount(async () => {
        console.log("running");

        // TODO: maybe auto-select current revision?
        await onRevisionClicked(sortedRevisions[0]);
    });

    function formatDate(ts: number): string {
        const date = new Date(ts);
        return date.toLocaleDateString() + " " + date.toLocaleTimeString();
    }

    async function onRevisionClicked(revision: YakManInstanceRevision) {
        if (revision.revision === selectedRevisionData.revision) {
            return; // Already selected
        }

        let oldRevision = selectedRevisionData.revision;

        // Set this here to avoid spamming requests if user spam click
        selectedRevisionData.revision = revision.revision;

        try {
            const { contentType, data } = await trpc(
                $page
            ).data.fetchRevisionData.query({
                configName: $page.params.config,
                instance: $page.params.instance,
                revision: revision.revision,
            });
            selectedRevisionData.contentType = contentType;
            selectedRevisionData.data = data;
            selectedRevisionData.labels = revision.labels.map(
                (l) => `${l.label_type}`
            );
        } catch (e) {
            console.error(e);
            // Since the update failed to get new data
            // rollback revision title to avoid confusing the user
            selectedRevisionData.revision = oldRevision;
        }
    }
</script>

<div class="flex justify-between gap-2">
    <div class="flex-grow mt-2">
        {#each sortedRevisions as revision}
            <div
                class={`flex justify-between my-2 p-1 px-2 rounded ${
                    revision.revision === selectedRevisionData.revision
                        ? "bg-yellow-200"
                        : "bg-gray-200"
                }`}
            >
                <div>
                    <!-- svelte-ignore a11y-click-events-have-key-events -->
                    <!-- svelte-ignore a11y-no-static-element-interactions -->
                    <div
                        class="inline-block text-blue-600 font-bold cursor-pointer"
                        on:click={() => onRevisionClicked(revision)}
                    >
                        {revision.revision}
                    </div>
                    {#if revision.revision == currentRevision}
                        (current)
                    {/if}
                </div>

                <p class="text-gray-700 text-sm">
                    {formatDate(revision.timestamp_ms)}
                </p>
            </div>
        {/each}
    </div>

    <div class="w-fit mr-2">
        <h1 class="text-lg font-bold mb-4">
            {`Revision: ${selectedRevisionData.revision}`}
        </h1>

        <YakManTextArea
            label="Data"
            value={selectedRevisionData.data}
            disabled
        />
        <div class="my-3">
            <YakManInput
                label="Content Type"
                value={selectedRevisionData.contentType}
                placeholder="my-config-name"
                disabled
            />
        </div>

        <div>
            <div class="block text-gray-700 text-sm font-bold mb-2">Labels</div>
            {#each selectedRevisionData.labels as label}
                <LabelPill text={label} />
            {/each}
            {#if selectedRevisionData.labels.length === 0}
                No labels
            {/if}
        </div>
    </div>
</div>
