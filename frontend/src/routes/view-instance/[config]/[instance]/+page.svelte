<script lang="ts">
    import { page } from "$app/stores";
    import LabelPill from "$lib/components/LabelPill.svelte";
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManSegmentSelect from "$lib/components/YakManSegmentSelect.svelte";
    import type { PageData } from "./$types";
    import ChangelogTab from "./ChangelogTab.svelte";
    import RevisionsTab from "./RevisionsTab.svelte";

    let { config, instance } = $page.params;

    export let data: PageData;

    let selectedHistoryTab: "Changelog" | "Revisions" = "Changelog";

    let sortedRevisions =
        data.revisions.sort((a, b) => b.timestamp_ms - a.timestamp_ms) ?? [];
    let sortedChangelog =
        data.instance?.changelog.sort(
            (a, b) => b.timestamp_ms - a.timestamp_ms
        ) ?? [];
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
        <YakManSegmentSelect
            bind:selectedOption={selectedHistoryTab}
            options={["Changelog", "Revisions"]}
        />

        {#if selectedHistoryTab == "Changelog"}
            <ChangelogTab {sortedChangelog} />
        {/if}

        {#if selectedHistoryTab == "Revisions"}
            <RevisionsTab
                {sortedRevisions}
                currentRevision={data.instance?.current_revision}
                pendingRevision={data.instance?.pending_revision}
            />
        {/if}
    </YakManCard>
</div>
