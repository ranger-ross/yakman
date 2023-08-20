<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import YakManTextArea from "$lib/components/YakManTextArea.svelte";
    import { trpc } from "$lib/trpc/client";
    import type { PageData } from "./$types";
    import LabelSelection from "./LabelSelection.svelte";
    import { openGlobaModal } from "$lib/stores/global-modal-state";

    const { config, instance } = $page.params;
    const editMode = !!instance;

    export let data: PageData;
    let { labels } = data;
    let selectedLabels: { [labelName: string]: string } = data.selectedLabels; // <LabelName, Value>
    let originalSelectedLabels = JSON.parse(
        JSON.stringify(data.selectedLabels)
    ); // Create a clone of the data

    let input = data.data?.data ?? "";
    let contentType = data.data?.contentType ?? "text/plain";

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
            Object.entries(selectedLabels).filter(([_, v]) => v != null)
        ) as { [labelName: string]: string };

        try {
            if (editMode) {
                await trpc($page).instances.updateConfigInstance.mutate({
                    configName: config,
                    instance: instance,
                    contentType: contentType,
                    data: input,
                    labels: filtedSelectedLabels,
                });
                goto(`/apply-changes/${config}/${instance}`);
            } else {
                const result = await trpc(
                    $page
                ).instances.createConfigInstance.mutate({
                    configName: config,
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
</script>

<div class="container mx-auto">
    <YakManCard>
        <h1 class="text-lg font-bold mb-4">
            {#if editMode}
                Edit Config Instance {config} -> {instance}
            {:else}
                Create Config Instance
            {/if}
        </h1>
        <YakManTextArea
            label="Data"
            bind:value={input}
            placeholder="My really cool config"
        />
        <div class="my-3">
            <YakManInput
                label="Content Type"
                bind:value={contentType}
                placeholder="my-config-name"
            />
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
