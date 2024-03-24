<script lang="ts">
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManLink from "$lib/components/YakManLink.svelte";
    import type { PageData } from "./$types";

    export let data: PageData;

    async function onDelete(teamId: string) {
        console.log(teamId);
    }
</script>

<YakManCard extraClasses="mt-2">
    <h2 class="text-xl font-bold">Teams</h2>
    <div class="flex-grow mt-2">
        <div class="bg-white rounded shadow-sm overflow-hidden">
            <table class="min-w-full divide-y divide-gray-200">
                <thead class="bg-gray-50">
                    <tr>
                        <th
                            scope="col"
                            class="px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider text-left"
                        >
                            Name
                        </th>
                        <th
                            scope="col"
                            class="px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider text-left"
                        >
                            ID
                        </th>
                        <th
                            scope="col"
                            class="px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider text-left"
                        >
                            Edit
                        </th>
                        <th
                            scope="col"
                            class="px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider text-left"
                        >
                            Delete
                        </th>
                    </tr>
                </thead>
                <tbody class="bg-white divide-y divide-gray-200">
                    {#each data.teams as team}
                        <tr>
                            <td class="px-6 py-2 whitespace-nowrap">
                                {team.name}
                            </td>
                            <td class="px-6 py-2 whitespace-nowrap text-sm">
                                {team.id}
                            </td>
                            <td class="px-6 py-2 whitespace-nowrap text-sm">
                                <YakManLink href="/teams/modify/{team.id}">
                                    Edit
                                </YakManLink>
                            </td>
                            <td class="px-6 py-2 whitespace-nowrap text-sm">
                                <YakManButton
                                    on:click={() => onDelete(team.id)}
                                    variant="danger"
                                >
                                    Delete
                                </YakManButton>
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    </div>
    <div class="mt-3">
        <YakManLink href="/teams/modify">Create Team</YakManLink>
    </div>
</YakManCard>
