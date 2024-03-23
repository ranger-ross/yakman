<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import { trpc } from "$lib/trpc/client";
    import type { PageData } from "./$types";
    import CopyableTextBlock from "./CopyableTextBlock.svelte";

    let users = ($page.data as PageData).users;
    let isOAuthEnabled = ($page.data as PageData).settings.enable_oauth;

    let newUsername = "";
    let resetPasswordLink: string | null = null;

    async function createUser() {
        for (const user of users) {
            if (user.email == newUsername) {
                console.log("user already added, skipping...");
                return;
            }
        }
        try {
            await trpc($page).users.createUser.mutate({
                username: newUsername,
                role: "Admin",
            });
            goto("/");
        } catch (e) {
            console.error(e);
        }
    }

    async function resetPassword(userId: string) {
        const { id, user_id } = await trpc(
            $page,
        ).auth.createResetPasswordLink.mutate({
            userId: userId,
        });

        const origin = $page.url.origin;
        resetPasswordLink = `${origin}/sessionuserIdassword?id=${id}&user_id=${user_id}`;
    }
</script>

<YakManCard extraClasses="mt-2">
    <h2 class="text-xl font-bold">Users</h2>
    <div class="flex-grow mt-2">
        <div class="bg-white rounded shadow-sm overflow-hidden">
            <table class="min-w-full divide-y divide-gray-200">
                <thead class="bg-gray-50">
                    <tr>
                        <th
                            scope="col"
                            class="px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider text-left"
                        >
                            Email
                        </th>
                        <th
                            scope="col"
                            class="px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider text-left"
                        >
                            User ID
                        </th>
                        {#if !isOAuthEnabled}
                            <th
                                scope="col"
                                class="px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider text-right"
                            >
                                Reset Password
                            </th>
                        {/if}
                    </tr>
                </thead>
                <tbody class="bg-white divide-y divide-gray-200">
                    {#each users as user}
                        <tr>
                            <td class="px-6 py-2 whitespace-nowrap">
                                {user.email}
                            </td>
                            <td class="px-6 py-2 whitespace-nowrap text-sm">
                                {user.id}
                            </td>
                            {#if !isOAuthEnabled}
                                <td
                                    class="px-6 py-2 whitespace-nowrap text-right"
                                >
                                    <p class="text-gray-700 text-sm">
                                        <YakManButton
                                            variant={"secondary"}
                                            on:click={() =>
                                                resetPassword(user.id)}
                                        >
                                            Reset Password
                                        </YakManButton>
                                    </p>
                                </td>
                            {/if}
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    </div>
</YakManCard>

{#if resetPasswordLink}
    <YakManCard extraClasses="mt-2">
        <CopyableTextBlock
            title="Reset Password Link"
            hint="Be sure to copy this link as it will not be shown again"
            text={resetPasswordLink}
        />
    </YakManCard>
{/if}

<YakManCard extraClasses="mt-2">
    <h2 class="text-xl font-bold">Add User</h2>
    <div class="flex items-end">
        <YakManInput placeholder="Username" bind:value={newUsername} />
        <YakManButton
            disabled={!newUsername || newUsername.length === 0}
            on:click={createUser}>Create user</YakManButton
        >
    </div>
</YakManCard>

