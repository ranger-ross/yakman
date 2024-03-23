<script lang="ts">
    import { page } from "$app/stores";
    import LabelPill from "$lib/components/LabelPill.svelte";
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import YakManTextArea from "$lib/components/YakManTextArea.svelte";
    import { trpc } from "$lib/trpc/client";
    import type { YakManInstanceRevision } from "$lib/types/types";
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { openGlobaModal } from "$lib/stores/global-modal-state";
    import MonacoEditor from "$lib/components/MonacoEditor.svelte";
    import ContentTypePill from "$lib/components/ContentTypePill.svelte";
    import { contentTypeToMonacoLanguage } from "$lib/utils/content-type-utils";

    export let config: string | null = null;
    export let instance: string | null = null;
    export let currentRevision: string | null = null;
    export let pendingRevision: string | null = null;
    export let sortedRevisions: YakManInstanceRevision[] = [];

    let selectedRevisionData = {
        revision: "",
        data: "",
        contentType: "",
        labels: [] as string[],
    };

    $: editorLanguage = contentTypeToMonacoLanguage(selectedRevisionData.contentType);

    onMount(async () => {
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
                $page,
            ).data.fetchRevisionData.query({
                configId: $page.params.config,
                instance: $page.params.instance,
                revision: revision.revision,
            });
            selectedRevisionData.contentType = contentType;
            selectedRevisionData.data = data;
            selectedRevisionData.labels = revision.labels.map(
                (l) => `${l.label_type}=${l.value}`,
            );
        } catch (e) {
            console.error(e);
            // Since the update failed to get new data
            // rollback revision title to avconfigIding the user
            selectedRevisionData.revision = oldRevision;
        }
    }

    async function deployRevision(revision: string) {
        if (pendingRevision === revision) {
            goto(`/apply-changes/${config}/${instance}`);
        } else {
            openGlobaModal({
                title: "Rollback to revision",
                message:
                    "Are you sure you want to rollback to this revision? " +
                    "A clone of this revision will be created and approval will be needed",
                async onConfirm() {
                    try {
                        await trpc(
                            $page,
                        ).revisions.rollbackInstanceRevision.mutate({
                            configId: config!,
                            instance: instance!,
                            revision: revision,
                        });
                        goto(`/apply-changes/${config}/${instance}`);
                    } catch (e) {
                        console.error(
                            "Error while rolling back reivision: ",
                            e,
                        );
                    }
                },
            });
        }
    }
</script>

<div class="flex justify-between gap-2">
    <div class="flex-grow mt-2">
        <div class="bg-white rounded shadow-sm overflow-hidden">
            <table class="min-w-full divide-y divide-gray-200">
                <thead class="bg-gray-50">
                    <tr>
                        <th
                            scope="col"
                            class="px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider text-left"
                            >Revision</th
                        >
                        <th
                            scope="col"
                            class="px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider text-right"
                            >Time</th
                        >
                    </tr>
                </thead>
                <tbody class="bg-white divide-y divide-gray-200">
                    {#each sortedRevisions as revision}
                        <tr
                            class={`${
                                revision.revision ===
                                selectedRevisionData.revision
                                    ? "bg-yellow-100"
                                    : ""
                            }`}
                        >
                            <td class="px-6 py-2 whitespace-nowrap">
                                <button
                                    class="inline-block text-blue-600 font-bold cursor-pointer"
                                    on:click={() => onRevisionClicked(revision)}
                                >
                                    {revision.revision}
                                </button>
                                {#if revision.revision == currentRevision}
                                    (current)
                                {/if}
                                {#if revision.revision == pendingRevision}
                                    (pending)
                                {/if}
                            </td>
                            <td class="px-6 py-2 whitespace-nowrap text-right">
                                <p class="text-gray-700 text-sm">
                                    {formatDate(revision.timestamp_ms)}
                                </p>
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    </div>

    <div class="w-fit mr-2 min-w-80">
        <h1 class="text-lg font-bold mb-1">
            {`Revision: ${selectedRevisionData.revision}`}
        </h1>

        <div class="h-48">
            <MonacoEditor
                content={selectedRevisionData.data}
                language={editorLanguage}
                disabled={true}
            />
        </div>

        <div class="my-3">
            <ContentTypePill contentType={selectedRevisionData.contentType}/>
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
        <div>
            <div class="block text-gray-700 text-sm font-bold my-2">
                Actions
            </div>
            {#each selectedRevisionData.labels as label}
                <LabelPill text={label} />
            {/each}
            <YakManButton
                on:click={() => deployRevision(selectedRevisionData.revision)}
                disabled={currentRevision === selectedRevisionData.revision}
            >
                Deploy
            </YakManButton>
        </div>
    </div>
</div>
