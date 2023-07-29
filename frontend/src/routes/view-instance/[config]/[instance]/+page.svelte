<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import LabelPill from "$lib/components/LabelPill.svelte";
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import { trpc } from "$lib/trpc/client";
    import type { YakManInstanceRevision } from "$lib/types/types";
    import type { PageData } from "./$types";

    let { config, instance } = $page.params;

    export let data: PageData;

    let sortedRevisions =
        data.revisions.sort((a, b) => b.timestamp_ms - a.timestamp_ms) ?? [];
    let sortedChangelog =
        data.instance?.changelog.sort(
            (a, b) => b.timestamp_ms - a.timestamp_ms
        ) ?? [];

    function formatDate(ts: number): string {
        const date = new Date(ts);
        return date.toLocaleDateString() + " " + date.toLocaleTimeString();
    }

    async function onRevisionClicked(revision: YakManInstanceRevision) {
        try {
            await trpc($page).revisions.updateInstanceRevision.mutate({
                configName: config,
                instance: instance,
                revision: revision.revision,
            });
            goto(`/apply-changes/${config}/${instance}`);
        } catch (e) {
            console.error("failed to update revision", e);
        }
    }
</script>

<div class="container mx-auto">
    <div class="mb-2">
        <YakManCard>
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-xl font-bold">{config}</h1>
                    <h1 class="text-md text-gray-700">{instance}</h1>
                </div>
                <div>
                    <a href={`/modify-instance/${config}/${instance}`}>
                        <YakManButton>Edit</YakManButton>
                    </a>
                </div>
            </div>
        </YakManCard>
    </div>
    <div class="mb-2">
        <YakManCard>
            <h1 class="text-lg font-bold mb-1">Content</h1>
            <div class="mb-2">
                <YakManCard>
                    <span class="font-bold mr-2">Content Type</span>
                    {data.data?.contentType}
                </YakManCard>
            </div>
            <div class="mb-2">
                <YakManCard>{data.data?.data}</YakManCard>
            </div>
        </YakManCard>
    </div>
    <div class="mb-2">
        <YakManCard>
            <h1 class="text-lg font-bold mb-1">Labels</h1>
            <div class="flex flex-wrap gap-2">
                {#if data.instance}
                    {#each data.instance.labels as label}
                        <LabelPill
                            text={`${label.label_type}=${label.value}`}
                        />
                    {/each}
                {/if}
            </div>
        </YakManCard>
    </div>
    <YakManCard>
        <h1 class="text-lg font-bold mb-1">History</h1>
        <h3 class="text-lg font-bold text-gray-800 mt-4">Revisions</h3>
        {#each sortedRevisions as revision}
            <div class="flex gap-2">
                <p>{formatDate(revision.timestamp_ms)} =></p>
                {#if revision.revision == data.instance?.current_revision}
                    <p class="text-yellow-400">{revision.revision}</p>
                {:else}
                    <p
                        class="text-blue-600 cursor-pointer"
                        on:click={() => onRevisionClicked(revision)}
                    >
                        {revision.revision}
                    </p>
                {/if}
                {#if revision.revision == data.instance?.pending_revision}
                    (pending)
                {/if}
            </div>
        {/each}
        <h3 class="text-lg font-bold text-gray-800 mt-4">Changelog</h3>
        {#each sortedChangelog as change}
            <div class="flex gap-2">
                <p>{formatDate(change.timestamp_ms)} =></p>
                {#if change.previous_revision}
                    <p>Previous: {change.previous_revision} =></p>
                {/if}
                <p>New: {change.new_revision}</p>
            </div>
        {/each}
    </YakManCard>
</div>
