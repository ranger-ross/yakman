<script lang="ts">
    import { page } from "$app/stores";
    import { replaceState } from "$app/navigation";
    import LabelPill from "$lib/components/LabelPill.svelte";
    import MonacoEditor from "$lib/components/MonacoEditor.svelte";
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManSegmentSelect from "$lib/components/YakManSegmentSelect.svelte";
    import { contentTypeToMonacoLanguage } from "$lib/utils/content-type-utils";
    import type { PageData } from "./$types";
    import ChangelogTab from "./ChangelogTab.svelte";
    import RevisionsTab from "./RevisionsTab.svelte";
    import ContentTypePill from "$lib/components/ContentTypePill.svelte";

    let { config, instance } = $page.params;

    export let data: PageData;
    $: editorLanguage = contentTypeToMonacoLanguage(data.data?.contentType);

    function onTabChange(option: string) {
        replaceState(`?tab=${option}`, {});
    }

    let selectedHistoryTab: "Changelog" | "Revisions" = data.tab ?? "Changelog";

    let sortedRevisions =
        data.revisions.sort((a, b) => b.timestamp_ms - a.timestamp_ms) ?? [];
    let sortedChangelog =
        data.instance?.changelog.sort(
            (a, b) => b.timestamp_ms - a.timestamp_ms,
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
                    <a
                        tabindex="-1"
                        href={`/modify-instance/${config}/${instance}`}
                    >
                        <YakManButton>Edit</YakManButton>
                    </a>
                </div>
            </div>
        </YakManCard>
    </div>
    <div class="mb-2">
        <YakManCard>
            <div class="flex gap-2">
                <h1 class="text-lg font-bold mb-1">Content</h1>
                <ContentTypePill contentType={data.data?.contentType} />
            </div>

            <div class="h-56 mt-2 mb-6">
                <MonacoEditor
                    content={data?.data?.data ?? ""}
                    language={editorLanguage}
                    disabled={true}
                />
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
                            text={`${data?.labels?.find((l) => l.id === label.label_id)?.name}=${label.value}`}
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
            on:select={(event) => onTabChange(event.detail)}
        />

        {#if selectedHistoryTab == "Changelog"}
            <ChangelogTab {sortedChangelog} users={data.users} />
        {/if}

        {#if selectedHistoryTab == "Revisions"}
            <RevisionsTab
                {config}
                {instance}
                {sortedRevisions}
                currentRevision={data.instance?.current_revision}
                pendingRevision={data.instance?.pending_revision}
            />
        {/if}
    </YakManCard>
</div>
