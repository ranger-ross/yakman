<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import YakManSelect from "$lib/components/YakManSelect.svelte";
    import { trpc } from "$lib/trpc/client";
    import type { PageData } from "./$types";

    export let data: PageData;

    let newUsername = "";
    let newApiKeyProject = data.projects[0].uuid;
    let newApiKeyRole = "Viewer";
    let newApiKey: string | null = null;

    async function createUser() {
        console.log("createUser");

        for (const user of data.users) {
            if (user.email == newUsername) {
                console.log("user already added, skipping...");
                return;
            }
        }
        try {
            await trpc($page).admin.createUser.mutate({
                username: newUsername,
                role: "Admin",
            });
            goto("/");
        } catch (e) {
            console.error(e);
        }
    }

    async function createApiKey() {
        console.log(newApiKeyProject, newApiKeyRole);

        const apiKey = await trpc($page).admin.createApiKey.mutate({
            projectUuid: newApiKeyProject,
            role: newApiKeyRole,
        });
        newApiKey = apiKey;
    }
</script>

<div class="container mx-auto">
    <h1 class="text-xl font-bold">Admin</h1>
    <YakManCard>
        <h2 class="text-xl font-bold">Users</h2>
        {#each data.users ?? [] as user}
            <li>{user.email}</li>
        {/each}
        <h2 class="text-xl font-bold">Add User</h2>
        Username
        <YakManInput placeholder="Username" bind:value={newUsername} />
        <br />
        <YakManButton on:click={createUser}>Create user</YakManButton>
    </YakManCard>

    <YakManCard extraClasses="mt-2">
        <h2 class="text-xl font-bold">Api Keys</h2>

        <table class="min-w-full divide-y divide-gray-200">
            <thead>
                <th>ID</th>
                <th>Project UUID</th>
                <th>Role</th>
                <th>Created At</th>
                <th>Created By</th>
            </thead>
            <tbody>
                {#each data.apiKeys ?? [] as apiKey}
                    <tr>
                        <td>{apiKey.id}</td>
                        <td>{apiKey.project_uuid}</td>
                        <td>{apiKey.role}</td>
                        <td>{apiKey.created_at}</td>
                        <td>{apiKey.created_by_uuid}</td>
                    </tr>
                {/each}
            </tbody>
        </table>

        <h2 class="text-xl font-bold mt-2">Create Api Key</h2>

        <YakManSelect label="Project" bind:value={newApiKeyProject}>
            {#each data.projects as project}
                <option value={project.uuid}>{project.name}</option>
            {/each}
        </YakManSelect>

        <YakManSelect label="Role" bind:value={newApiKeyRole}>
            <option value="Viewer">Viewer</option>
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
</div>
