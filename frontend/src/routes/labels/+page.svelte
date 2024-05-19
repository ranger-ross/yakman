<script lang="ts">
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManLink from "$lib/components/YakManLink.svelte";
    import type { PageData } from "./$types";

    export let data: PageData;

    console.log(data);

    async function handleDeleteLabel() {
        console.log("TODO: Delete");
    }
</script>

<div class="container mx-auto">
    <YakManCard>
        <h1 class="text-lg font-bold mb-4">Labels</h1>

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
                        Description
                    </th>
                    <th
                        scope="col"
                        class="px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider text-left"
                    >
                        Options
                    </th>
                    <th
                        scope="col"
                        class="px-6 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider text-right"
                    />
                </tr>
            </thead>
            <tbody class="bg-white divide-y divide-gray-200">
                {#each data.labels ?? [] as label}
                    <tr>
                        <td class="px-6 py-2 whitespace-nowrap">
                            {label.name}
                        </td>
                        <td class="px-6 py-2 whitespace-nowrap text-sm">
                            {label.description}
                        </td>

                        <td class="px-6 py-2 whitespace-nowrap text-sm">
                            {label.options}
                        </td>
                        <td class="px-6 py-2 whitespace-nowrap text-sm">
                            <YakManLink href={`/labels/edit/${label.name}`}>
                                Edit
                            </YakManLink>
                        </td>
                        <td class="px-6 py-2 whitespace-nowrap text-sm">
                            <YakManButton
                                variant="danger"
                                on:click={handleDeleteLabel}
                            >
                                Delete
                            </YakManButton>
                        </td>
                    </tr>
                {/each}
            </tbody>
        </table>
    </YakManCard>
    <YakManCard extraClasses="mt-2">
        <YakManLink href="/labels/edit">Create Label</YakManLink>
    </YakManCard>
</div>
