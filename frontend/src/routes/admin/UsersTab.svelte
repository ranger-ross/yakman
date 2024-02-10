<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import { trpc } from "$lib/trpc/client";

    export let users: any[] = [];

    let newUsername = "";
    let resetPasswordUserUuid = "";
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
</script>

<YakManCard extraClasses="mt-2">
    <h2 class="text-xl font-bold mt-2">Users</h2>
    {#each users as user}
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
    <YakManInput placeholder="User UUID" bind:value={resetPasswordUserUuid} />

    {#if resetPasswordLink}
        <div class="text-lg my-3">
            {resetPasswordLink}
        </div>
    {/if}

    <YakManButton on:click={resetPassword}>Reset Password</YakManButton>
</YakManCard>
