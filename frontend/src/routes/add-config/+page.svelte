<script lang="ts">
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import YakManSelect from "$lib/components/YakManSelect.svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { trpc } from "$lib/trpc/client";
    import type { PageData } from "./$types";
    import { TRPCClientError } from "@trpc/client";

    export let data: PageData;

    let name = "";
    let defaultProjectId = $page.url.searchParams.get("project");
    let defaultProject =
        data.projects.find((p) => p.id === defaultProjectId) ??
        data.projects[0];
    let selectedProjectId = defaultProject.id;
    let error: string | null = null;
    async function onCreateConfig() {
        try {
            await trpc($page).configs.createConfig.mutate({
                name: name,
                projectId: selectedProjectId,
            });
            goto(`/?project=${selectedProjectId}`);
        } catch (e) {
            if (e instanceof TRPCClientError) {
                let errorData = JSON.parse(e.message);
                error = errorData?.message ?? "An error occured";
            }
        }
    }
</script>

<div class="container mx-auto">
    <YakManCard>
        <h1 class="text-lg font-bold mb-4">Add Config</h1>
        <div class="mb-3">
            <YakManInput
                label="Name"
                bind:value={name}
                placeholder="my-config-name"
                mask="kebab-case"
            />
        </div>
        <div class="mb-3">
            <YakManSelect label="Project" bind:value={selectedProjectId}>
                {#each data.projects as project}
                    <option value={project.id}>{project.name}</option>
                {/each}
            </YakManSelect>
        </div>
        <div class="text-red-500 font-bold mb-1">
            {#if error}
                Error: {error}
            {/if}
        </div>
        <YakManButton
            on:click={onCreateConfig}
            disabled={!name || name.length === 0}>Create</YakManButton
        >
    </YakManCard>
</div>
