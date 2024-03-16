<script lang="ts">
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import { page } from "$app/stores";
    import { trpc } from "$lib/trpc/client";
    import { goto } from "$app/navigation";
    import type { PageData } from "./$types";

    export let data: PageData;

    let projectId = $page.params.uuid;
    const isNewProject = !projectId;
    let name = data.project?.name ?? "";

    async function onSave() {
        if (isNewProject) {
            onCreateProject();
        } else {
            // TODO: Implment
            console.error("UPDATE PROJECT NOT IMPLMENTED");
        }
    }

    async function onCreateProject() {
        try {
            const { projectUuid } =
                await trpc($page).projects.createProject.mutate(name);
            goto(`/?project=${projectUuid}`);
        } catch (e) {
            console.error("Error creating project", e);
        }
    }
</script>

<div class="container mx-auto">
    <YakManCard>
        <h1 class="text-lg font-bold mb-4">
            {#if isNewProject}
                Add Project
            {:else}
                Modify Project
            {/if}
        </h1>
        <div class="mb-3">
            <YakManInput
                label="Name"
                placeholder="my-project"
                bind:value={name}
                disabled={!isNewProject}
                mask="kebab-case"
            />
        </div>
    </YakManCard>

    <YakManCard extraClasses="mt-2">
        <YakManButton
            on:click={onSave}
            type="submit"
            disabled={!name || name.length === 0}
        >
            {#if isNewProject}
                Create
            {:else}
                Update
            {/if}
        </YakManButton>
    </YakManCard>
</div>
