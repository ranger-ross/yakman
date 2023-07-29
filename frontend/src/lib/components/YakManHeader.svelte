<script lang="ts">
    import { roles } from "$lib/stores/roles";

    let isAdmin = false;

    roles.subscribe((value) => {
        isAdmin = value?.globalRoles?.includes("Admin") ?? false;
    });
</script>

<div
    class="bg-white shadow-sm h-14 flex justify-end items-center gap-3 mb-2 p-2"
>
    <a class="text-2xl font-bold" href="/">YakMan</a>
    <div style="flex-grow: 1" />
    <a href="/login">Login</a>
    <a href="/add-config">Add Config</a>
    <a href="/add-label">Add Label</a>
    {#if isAdmin}
        <a href="/add-project">Add Project</a>
        <a href="/admin">Admin</a>
    {/if}
</div>
