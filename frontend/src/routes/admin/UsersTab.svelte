<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import { trpc } from "$lib/trpc/client";
    import type { PageData } from "./$types";

    let users = ($page.data as PageData).users;

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
            await trpc($page).admin.createUser.mutate({
                username: newUsername,
                role: "Admin",
            });
            goto("/");
        } catch (e) {
            console.error(e);
        }
    }

    async function resetPassword(userUuid: string) {
        const { id, user_uuid } = await trpc(
            $page,
        ).auth.createResetPasswordLink.mutate({
            userUuid: userUuid,
        });

        const origin = $page.url.origin;
        resetPasswordLink = `${origin}/session/reset-password?id=${id}&user_uuid=${user_uuid}`;
    }
</script>

<YakManCard extraClasses="mt-2">
    <h2 class="text-xl font-bold mt-2">Users</h2>
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
                        <th
                            scope="col"
                            class="px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider text-right"
                        >
                            Reset Password
                        </th>
                    </tr>
                </thead>
                <tbody class="bg-white divide-y divide-gray-200">
                    {#each users as user}
                        <tr>
                            <td class="px-6 py-2 whitespace-nowrap">
                                {user.email}
                            </td>
                            <td class="px-6 py-2 whitespace-nowrap text-sm">
                                {user.uuid}
                            </td>
                            <td class="px-6 py-2 whitespace-nowrap text-right">
                                <p class="text-gray-700 text-sm">
                                    <YakManButton
                                        variant={"secondary"}
                                        on:click={() =>
                                            resetPassword(user.uuid)}
                                    >
                                        Reset Password
                                    </YakManButton>
                                </p>
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    </div>
</YakManCard>

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
