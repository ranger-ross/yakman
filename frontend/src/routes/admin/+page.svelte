<script lang="ts">
    import { goto, invalidateAll, replaceState } from "$app/navigation";
    import { page } from "$app/stores";
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import YakManSegmentSelect from "$lib/components/YakManSegmentSelect.svelte";
    import YakManSelect from "$lib/components/YakManSelect.svelte";
    import { trpc } from "$lib/trpc/client";
    import type { PageData } from "./$types";

    export let data: PageData;

    let newUsername = "";
    let resetPasswordUserUuid = "";
    let newApiKeyProject = data.projects[0].uuid;
    let newApiKeyRole = "Viewer";
    let newApiKey: string | null = null;
    let resetPasswordLink: string | null = null;

    let selectedHistoryTab: "Users" | "Api Keys" = data.tab ?? "Users";

    function onTabChange(option: string) {
        replaceState(`?tab=${option}`, {});
    }

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

    async function resetPassword() {
        console.log(resetPasswordUserUuid);

        const { id, user_uuid } = await trpc(
            $page,
        ).auth.createResetPasswordLink.mutate({
            userUuid: resetPasswordUserUuid,
        });

        const origin = $page.url.origin;
        resetPasswordLink = `${origin}/session/reset-password?id=${id}&user_uuid=${user_uuid}`;
    }

    async function deleteApiKey(id: string) {
        await trpc($page).admin.deleteApiKey.mutate({
            id: id,
        });
        invalidateAll();
    }
</script>

<div class="container mx-auto">
    <h1 class="text-xl font-bold">Admin</h1>

    <YakManCard>
        <YakManSegmentSelect
            bind:selectedOption={selectedHistoryTab}
            options={["Users", "Api Keys"]}
            on:select={(event) => onTabChange(event.detail)}
        />
    </YakManCard>

    {#if selectedHistoryTab == "Users"}
        <YakManCard extraClasses="mt-2">
            <h2 class="text-xl font-bold mt-2">Users</h2>
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
            <h2 class="text-xl font-bold mt-2">Reset Password</h2>
            <YakManInput
                placeholder="User UUID"
                bind:value={resetPasswordUserUuid}
            />

            {#if resetPasswordLink}
                <div class="text-lg my-3">
                    {resetPasswordLink}
                </div>
            {/if}

            <YakManButton on:click={resetPassword}>Reset Password</YakManButton>
        </YakManCard>
    {/if}

    {#if selectedHistoryTab == "Api Keys"}
        <YakManCard extraClasses="mt-2">
            <h2 class="text-xl font-bold">Api Keys</h2>

            <table class="min-w-full divide-y divide-gray-200">
                <thead>
                    {#each ["ID", "Project", "Role", "Created By", "Created At", ""] as col}
                        <th class="text-left">{col}</th>
                    {/each}
                </thead>
                <tbody>
                    {#each data.apiKeyTableRows ?? [] as apiKey}
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
                                <YakManButton
                                    on:click={() => deleteApiKey(apiKey.id)}
                                >
                                    Delete
                                </YakManButton>
                            </td>
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
                <option value="Operator">Operator</option>
                <option value="Approver">Approver</option>
                <option value="Admin">Admin</option>
            </YakManSelect>

            <div class="mt-2">
                <YakManButton on:click={createApiKey}
                    >Create Api Key</YakManButton
                >
            </div>

            {#if newApiKey}
                <div>
                    New Api Key
                    <YakManInput disabled value={newApiKey ?? ""} />
                    Be sure to copy this key as it will not be shown again.
                </div>
            {/if}
        </YakManCard>
    {/if}
</div>
