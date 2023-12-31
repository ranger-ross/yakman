<script lang="ts">
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import YakManSelect from "$lib/components/YakManSelect.svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { trpc } from "$lib/trpc/client";
    import type { PageData } from "./$types";

    export let data: PageData;

    let name = "";
    let defaultProjectUuid = $page.url.searchParams.get("project");
    let defaultProject =
        data.projects.find((p) => p.uuid === defaultProjectUuid) ??
        data.projects[0];
    let selectedProjectUuid = defaultProject.uuid;

    async function onCreateConfig() {
        try {
            await trpc($page).configs.createConfig.mutate({
                name: name,
                projectUuid: selectedProjectUuid,
            });
            goto(`/?project=${selectedProjectUuid}`);
        } catch (e) {
            console.error("Error creating config:", e);
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
            <YakManSelect label="Project" bind:value={selectedProjectUuid}>
                {#each data.projects as project}
                    <option value={project.uuid}>{project.name}</option>
                {/each}
            </YakManSelect>
        </div>
        <YakManButton
            on:click={onCreateConfig}
            disabled={!name || name.length === 0}>Create</YakManButton
        >
    </YakManCard>
</div>
