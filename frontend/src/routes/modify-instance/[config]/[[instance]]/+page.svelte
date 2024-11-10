<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import { trpc } from "$lib/trpc/client";
    import type { PageData } from "./$types";
    import LabelSelection from "./LabelSelection.svelte";
    import { openGlobaModal } from "$lib/stores/global-modal-state";
    import YakManAutoComplete from "$lib/components/YakManAutoComplete.svelte";
    import MonacoEditor from "$lib/components/MonacoEditor.svelte";
    import { contentTypeToMonacoLanguage } from "$lib/utils/content-type-utils";

    const { config, instance } = $page.params;
    const editMode = !!instance;

    export let data: PageData;
    let { labels } = data;
    let selectedLabels: { [labelId: string]: string } = data.selectedLabels; // <LabelId, Value>
    let originalSelectedLabels = structuredClone(data.selectedLabels);
    let input = data.data?.data ?? "";
    let contentType = data.data?.contentType ?? "text/plain";
    $: editorLanguage = contentTypeToMonacoLanguage(contentType);

    function onSubmit() {
        const title = editMode ? "Update Config" : "Create Config";
        const message = editMode
            ? "Are you sure you want to update this config? (approval will be required before it takes effect)"
            : "Are you sure you want to create this config?";

        openGlobaModal({
            title: title,
            message: message,
            onConfirm() {
                saveChanges();
            },
        });
    }

    async function saveChanges() {
        // Remove any non-selected labels
        const filtedSelectedLabels = Object.fromEntries(
            Object.entries(selectedLabels).filter(([_, v]) => v != null),
        ) as { [labelName: string]: string };

        console.log(filtedSelectedLabels);

        try {
            if (editMode) {
                await trpc($page).instances.updateConfigInstance.mutate({
                    configId: config,
                    instance: instance,
                    contentType: contentType,
                    data: input,
                    labels: filtedSelectedLabels,
                });
                goto(`/apply-changes/${config}/${instance}`);
            } else {
                const result = await trpc(
                    $page,
                ).instances.createConfigInstance.mutate({
                    configId: config,
                    contentType: contentType,
                    data: input,
                    labels: filtedSelectedLabels,
                });
                goto(`/view-instance/${config}/${result.instance}`);
            }
        } catch (e) {
            console.error(e);
        }
    }

    function onDeleteClicked() {
        openGlobaModal({
            title: "Are you sure you want to delete this instance?",
            message: "This cannot be undone.",
            confirmButtonVariant: "danger",
            confirmButtonText: "Delete",
            async onConfirm() {
                await trpc($page).instances.deleteConfigInstance.mutate({
                    configId: config,
                    instance: instance,
                });

                goto("/");
            },
        });
    }

    let hasChanges = false;
    $: {
        hasChanges = (() => {
            if (input !== data.data?.data) {
                return true;
            }
            if (contentType !== data.data?.contentType) {
                return true;
            }

            if (
                Object.keys(originalSelectedLabels).length !==
                Object.keys(selectedLabels).length
            ) {
                return true;
            }

            for (const key of Object.keys(originalSelectedLabels)) {
                if (originalSelectedLabels[key] !== selectedLabels[key]) {
                    return true;
                }
            }
            return false;
        })();
    }

    function formatJSON(value: string): string {
        return JSON.stringify(JSON.parse(value), null, 2);
    }

    let canFormat = false;
    $: {
        canFormat = (() => {
            if (contentType?.toLowerCase().includes("json")) {
                try {
                    const j = JSON.parse(input);

                    if (formatJSON(input) === input) {
                        return false;
                    }
                    return !!j;
                } catch {
                    return false;
                }
            }

            return false;
        })();
    }
    function format() {
        if (contentType?.toLowerCase().includes("json")) {
            input = formatJSON(input);
        }
    }
</script>

<div class="container mx-auto">
    <YakManCard>
        <div class="flex justify-between">
            <h1 class="text-lg font-bold mb-4">
                {#if editMode}
                    Edit Config Instance {config} -> {instance}
                {:else}
                    Create Config Instance
                {/if}
            </h1>
            {#if editMode}
                <div>
                    <YakManButton variant="danger" on:click={onDeleteClicked}>
                        Delete
                    </YakManButton>
                </div>
            {/if}
        </div>

        <div class="h-56">
            <label class="block text-gray-700 text-sm font-bold mb-2">
                Data
            </label>
            <MonacoEditor bind:content={input} language={editorLanguage} />
        </div>

        <div class="my-8 flex justify-between">
            <YakManAutoComplete
                label="Content Type"
                placeholder="application/json"
                bind:value={contentType}
                options={[
                    "application/json",
                    "application/yaml",
                    "text/plain",
                    "text/html",
                ]}
            />

            <div>
                <YakManButton on:click={format} disabled={!canFormat}>
                    Format
                </YakManButton>
            </div>
        </div>
        <LabelSelection {labels} bind:selectedLabels />
        <YakManButton on:click={onSubmit} disabled={!hasChanges}>
            {#if editMode}
                Update
            {:else}
                Create
            {/if}
        </YakManButton>
    </YakManCard>
</div>
