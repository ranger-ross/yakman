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

    const { config, instance } = $page.params;
    const editMode = !!instance;

    export let data: PageData;
    let { labels } = data;
    let selectedLabels: { [labelName: string]: string } = data.selectedLabels; // <LabelName, Value>

    let input = data.data?.data ?? "";
    let contentType = data.data?.contentType ?? "text/plain";

    async function onSubmit() {
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
        <YakManButton on:click={onSubmit}>
            {#if editMode}
                Update
            {:else}
                Create
            {/if}
        </YakManButton>
    </YakManCard>
</div>
