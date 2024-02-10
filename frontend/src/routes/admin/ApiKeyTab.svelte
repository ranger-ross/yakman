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
        <thead class="bg-gray-50">
            <tr>
                {#each ["ID", "Project", "Role", "Created By", "Created At", ""] as col}
                    <th
                        scope="col"
                        class="px-3 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider text-left"
                    >
                        {col}
                    </th>
                {/each}
            </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-200">
            {#each apiKeyTableRows as apiKey}
                <tr>
                    <td class="px-3 py-2 whitespace-nowrap text-sm">
                        {apiKey.id}
                    </td>
                    <td class="px-3 py-2 whitespace-nowrap text-sm">
                        {apiKey.projectName}
                    </td>
                    <td class="px-3 py-2 whitespace-nowrap text-sm">
                        {apiKey.role}
                    </td>
                    <td class="px-3 py-2 whitespace-nowrap text-sm">
                        {apiKey.createdBy}
                    </td>
                    <td
                        class="px-3 py-2 whitespace-nowrap text-sm text-gray-500"
                    >
                        {apiKey.createdAt.toLocaleDateString()}
                        {apiKey.createdAt.toLocaleTimeString()}
                    </td>
                    <td>
                        <YakManButton
                            on:click={() => deleteApiKey(apiKey.id)}
                            variant={"secondary"}
                        >
                            Delete
                        </YakManButton>
                    </td>
                </tr>
            {/each}
        </tbody>
    </table>
</YakManCard>

<YakManCard extraClasses="mt-2">
    <h2 class="text-xl font-bold">Create Api Key</h2>

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
