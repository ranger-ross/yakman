<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import { trpc } from "$lib/trpc/client";
    import type { PageData } from "./$types";

    export let data: PageData;

    const isNewTeam = !data.team;
    let teamName = data.team?.name ?? '';

    async function createTeam() {
        if (isNewTeam) {
            await trpc($page).teams.createTeam.mutate({
                name: teamName,
            });
            goto('/teams');
        } else {
            console.warn("TODO: Add ability to update team");
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
        <YakManInput label="Name" bind:value={teamName} mask="kebab-case" />

        TODO: Add ability to grant access
    </div>

    <YakManButton disabled={isInvalid} on:click={createTeam}>
        {#if isNewTeam}
            Create Team
        {:else}
            Update Team
        {/if}
    </YakManButton>
</YakManCard>
