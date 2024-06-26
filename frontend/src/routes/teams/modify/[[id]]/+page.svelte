<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import YakManSelect from "$lib/components/YakManSelect.svelte";
    import { trpc } from "$lib/trpc/client";
    import type { YakManRole } from "$lib/types/types";
    import type { PageData } from "./$types";
    import MultiSelect from "svelte-multiselect";

    const ROLE_OPTIONS: YakManRole[] = [
        "Viewer",
        "Operator",
        "Approver",
        "Admin",
    ];

    export let data: PageData;

    const isNewTeam = !data.team;
    let teamName = data.team?.name ?? "";
    let globalRole: YakManRole | undefined =
        data.team?.global_roles[0] ?? undefined;
    let selectedProjects: { label: string; value: string }[] = (() => {
        let projectsWithRoles = data.team?.roles.map((role) => role.project_id);
        return data.projects
            .filter((p) => projectsWithRoles?.includes(p.id))
            .map((project) => ({
                label: project.name,
                value: project.id,
            }));
    })();
    let projectRoles: { [projectId: string]: YakManRole | undefined } = (() => {
        const output: { [projectId: string]: YakManRole | undefined } = {};

        data.team?.roles.forEach((role) => {
            output[role.project_id] = role.role;
        });

        return output;
    })();
    let teamMembers: { label: string; value: string }[] = (() => {
        const output: { label: string; value: string }[] = [];

        for (let userId of data.team?.member_user_ids ?? []) {
            let email = data.users.find((u) => u.id === userId)?.email;
            output.push({
                value: userId,
                label: email ?? userId,
            });
        }

        return output;
    })();

    console.log(data);

    async function createTeam() {
        const roles = Object.keys(projectRoles)
            .filter((projectId) => !!projectRoles[projectId])
            .map((projectId) => ({
                projectId,
                role: projectRoles[projectId]!,
            }));

        const teamMemberUserIds = teamMembers.map((member) => member.value);

        if (isNewTeam) {
            await trpc($page).teams.createTeam.mutate({
                name: teamName,
                globalRole: globalRole,
                roles: roles,
                teamMembers: teamMemberUserIds,
            });
            goto("/teams");
        } else {
            await trpc($page).teams.updateTeam.mutate({
                teamId: data.team!.id,
                name: teamName,
                globalRole: globalRole,
                roles: roles,
                teamMembers: teamMemberUserIds,
            });
            goto("/teams");
        }
    }

    $: isInvalid = (() => {
        if (!teamName || teamName.length === 0) {
            return true;
        }
        return false;
    })();
</script>

<YakManCard>
    <h1 class="text-lg font-bold mb-4">Teams</h1>

    <div class="mb-3">
        <YakManInput
            label="Name"
            bind:value={teamName}
            mask="kebab-case"
            containerClass="w-64 mb-2"
        />
    </div>
</YakManCard>

<YakManCard extraClasses="mt-2">
    <YakManSelect label="Global Role" bind:value={globalRole}>
        <option value={undefined}>None</option>
        {#each ROLE_OPTIONS as role}
            <option value={role}>{role}</option>
        {/each}
    </YakManSelect>

    <label
        class="block text-gray-700 text-sm font-bold mb-2 mt-4"
        for="ProjectRoles"
    >
        Project Roles
    </label>

    <div class="w-96">
        <MultiSelect
            name="ProjectRoles"
            placeholder="Select project to add roles"
            bind:selected={selectedProjects}
            options={data.projects.map((p) => ({
                label: p.name,
                value: p.id,
            }))}
        />
    </div>

    {#each selectedProjects as project}
        <div class="mt-2">
            <YakManSelect
                label="{project.label} Project Role"
                bind:value={projectRoles[project.value]}
            >
                <option value={undefined}>None</option>
                {#each ROLE_OPTIONS as role}
                    <option value={role}>{role}</option>
                {/each}
            </YakManSelect>
        </div>
    {/each}
</YakManCard>

<YakManCard extraClasses="mt-2">
    <label
        class="block text-gray-700 text-sm font-bold mb-2"
        for="ProjectRoles"
    >
        Team Members
    </label>

    <div class="w-96">
        <MultiSelect
            name="TeamMembers"
            bind:selected={teamMembers}
            placeholder="Select project to add roles"
            options={data.users.map((user) => ({
                label: user.email,
                value: user.id,
            }))}
        />
    </div>
</YakManCard>

<YakManCard extraClasses="mt-2">
    <YakManButton disabled={isInvalid} on:click={createTeam}>
        {#if isNewTeam}
            Create Team
        {:else}
            Update Team
        {/if}
    </YakManButton>
</YakManCard>
