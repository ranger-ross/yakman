<script lang="ts">
    import { invalidateAll } from "$app/navigation";
    import { page } from "$app/stores";
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import YakManSelect from "$lib/components/YakManSelect.svelte";
    import { trpc } from "$lib/trpc/client";
    import type { PageData } from "./$types";

    let apiKeyTableRows = ($page.data as PageData).apiKeyTableRows;
    let projects = ($page.data as PageData).projects;
    let newApiKeyProject = projects[0].uuid;
    let newApiKeyRole = "Viewer";
    let newApiKey: string | null = null;

    async function createApiKey() {
        console.log(newApiKeyProject, newApiKeyRole);

        const apiKey = await trpc($page).admin.createApiKey.mutate({
            projectUuid: newApiKeyProject,
            role: newApiKeyRole,
        });
        newApiKey = apiKey;
    }

    async function deleteApiKey(id: string) {
        await trpc($page).admin.deleteApiKey.mutate({
            id: id,
        });
        invalidateAll();
    }
</script>

<YakManCard extraClasses="mt-2">
    <h2 class="text-xl font-bold">Api Keys</h2>

    <table class="min-w-full divide-y divide-gray-200">
        <thead>
            {#each ["ID", "Project", "Role", "Created By", "Created At", ""] as col}
                <th class="text-left">{col}</th>
            {/each}
        </thead>
        <tbody>
            {#each apiKeyTableRows ?? [] as apiKey}
                <tr>
                    <td>{apiKey.id}</td>
                    <td>{apiKey.projectName}</td>
                    <td>{apiKey.role}</td>
                    <td>{apiKey.createdBy}</td>
                    <td>
                        {apiKey.createdAt.toLocaleDateString()}
                        {apiKey.createdAt.toLocaleTimeString()}
                    </td>
                    <td>
                        <YakManButton on:click={() => deleteApiKey(apiKey.id)}>
                            Delete
                        </YakManButton>
                    </td>
                </tr>
            {/each}
        </tbody>
    </table>

    <h2 class="text-xl font-bold mt-2">Create Api Key</h2>

    <YakManSelect label="Project" bind:value={newApiKeyProject}>
        {#each projects as project}
            <option value={project.uuid}>{project.name}</option>
        {/each}
    </YakManSelect>

    <YakManSelect label="Role" bind:value={newApiKeyRole}>
        <option value="Viewer">Viewer</option>
        <option value="Operator">Operator</option>
        <option value="Approver">Approver</option>
        <option value="Admin">Admin</option>
    </YakManSelect>

    <div class="mt-2">
        <YakManButton on:click={createApiKey}>Create Api Key</YakManButton>
    </div>

    {#if newApiKey}
        <div>
            New Api Key
            <YakManInput disabled value={newApiKey ?? ""} />
            Be sure to copy this key as it will not be shown again.
        </div>
    {/if}
</YakManCard>
