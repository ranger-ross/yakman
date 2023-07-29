<script lang="ts">
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import { page } from "$app/stores";
    import { trpc } from "$lib/trpc/client";
    import { goto } from "$app/navigation";

    let name = "";

    async function onCreateProject() {
        try {
            await trpc($page).projects.createProject.mutate(name);
            goto("/");
        } catch (e) {
            console.error("Error creating project", e);
        }
    }
</script>

<div class="container mx-auto">
    <YakManCard>
        <h1 class="text-lg font-bold mb-4">Add Project</h1>
        <div class="mb-3">
            <YakManInput
                label="Name"
                placeholder="my-project"
                bind:value={name}
                mask="kebab-case"
            />
        </div>
        <YakManButton on:click={onCreateProject} type="submit">
            Create
        </YakManButton>
    </YakManCard>
</div>
