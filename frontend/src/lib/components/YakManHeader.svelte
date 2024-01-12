<script lang="ts">
    import { goto } from "$app/navigation";
    import ProfileIcon from "$lib/icons/ProfileIcon.svelte";
    import { roles } from "$lib/stores/roles";
    import { userInfo } from "$lib/stores/user-info";
    import { hasRoles } from "$lib/utils/role-utils";
    import YakManPopoverMenu from "./YakManPopoverMenu.svelte";

    let isLoggedIn = false;
    let isAdmin = false;
    let profilePictureUrl: string | null;

    roles.subscribe((value) => {
        isLoggedIn = hasRoles(value.globalRoles, value.roles)
        isAdmin = value?.globalRoles?.includes("Admin") ?? false;
    });

    userInfo.subscribe((value) => {
        profilePictureUrl = value.profilePictureUrl ?? null;
    });

    async function onLogout() {
        await fetch("/session/logout", { method: "POST" });
        goto(`/login`);
    }
</script>

<div
    class="bg-white shadow-sm h-14 flex justify-between items-center gap-3 mb-2 p-2"
>
    <a class="text-2xl font-bold" href="/">YakMan</a>

    {#if isLoggedIn}
        <YakManPopoverMenu
            options={[
                { text: "Add Label", value: "AddLabel" },
                ...(isAdmin
                    ? [
                          { text: "Add Project", value: "AddProject" },
                          { text: "Admin", value: "Admin" },
                      ]
                    : []),
                { text: "Logout", value: "Logout" },
            ]}
            on:select={(value) => {
                const selection = value.detail;
                switch (true) {
                    case selection === "AddLabel":
                        return goto(`/add-label`);
                    case selection === "AddProject":
                        return goto(`/add-project`);
                    case selection === "Admin":
                        return goto(`/admin`);
                    case selection === "Logout":
                        return onLogout();
                }
            }}
        >
            {#if profilePictureUrl}
                <img
                    class="rounded-full h-10 w-10 object-cover"
                    alt="menu"
                    src={profilePictureUrl}
                />
            {:else}
                <ProfileIcon />
            {/if}
        </YakManPopoverMenu>
    {/if}
</div>
