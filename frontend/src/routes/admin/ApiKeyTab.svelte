<script lang="ts">
    import { invalidateAll } from "$app/navigation";
    import { page } from "$app/stores";
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManSelect from "$lib/components/YakManSelect.svelte";
    import { trpc } from "$lib/trpc/client";
    import type { PageData } from "./$types";

    let apiKeyTableRows = ($page.data as PageData).apiKeyTableRows;
    let projects = ($page.data as PageData).projects;

    $: {
        apiKeyTableRows = ($page.data as PageData).apiKeyTableRows;
        projects = ($page.data as PageData).projects;
    }

    let newApiKeyProject = projects[0].uuid;
    let newApiKeyRole = "Viewer";
    let newApiKey: string | null = null;
    let copied = false;

    async function createApiKey() {
        console.log(newApiKeyProject, newApiKeyRole);

        const apiKey = await trpc($page).apiKeys.createApiKey.mutate({
            projectUuid: newApiKeyProject,
            role: newApiKeyRole,
        });
        newApiKey = apiKey;
        invalidateAll();
    }

    async function deleteApiKey(id: string) {
        await trpc($page).apiKeys.deleteApiKey.mutate({
            id: id,
        });
        invalidateAll();
    }

    function copyApiKey() {
        navigator.clipboard.writeText(newApiKey!!);
        copied = true;
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
        <div class="mt-2">
            <div class="bg-gray-50 text-white p-4 rounded-md">
                <div class="flex justify-between items-center mb-2">
                    <span class="text-gray-500">New Api Key</span>
                    <button
                        class="code bg-gray-200 hover:bg-gray-300 text-gray-500 px-3 py-1 rounded-md"
                        data-clipboard-target="#code"
                        on:click={copyApiKey}
                    >
                        {#if copied}
                            Copied
                        {:else}
                            Copy
                        {/if}
                    </button>
                </div>
                <div class="overflow-x-auto">
                    <code class="text-gray-800">
                        {newApiKey ?? ""}
                    </code>
                </div>
            </div>
            <p class="text-gray-600">
                Be sure to copy this key as it will not be shown again.
            </p>
        </div>
    {/if}
</YakManCard>
