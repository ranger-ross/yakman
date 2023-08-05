<script lang="ts">
    import type { ConfigInstanceChange } from "$lib/types/types";

    export let sortedChangelog: ConfigInstanceChange[] = [];

    function formatDate(ts: number): string {
        const date = new Date(ts);
        return date.toLocaleDateString() + " " + date.toLocaleTimeString();
    }
</script>

<div class="mt-2">
    {#each sortedChangelog as change, index}
        <div
            class={`grid grid-cols-2 my-2 p-1 px-2 rounded ${
                index % 2 == 0 ? "bg-gray-200" : "bg-gray-100"
            }`}
        >
            <div>
                <p class="font-bold">
                    {#if change.previous_revision}
                        Updated
                    {:else}
                        Created
                    {/if}
                </p>

                <p class="text-sm text-gray-500">
                    {formatDate(change.timestamp_ms)}
                </p>
            </div>

            <div>
                {#if change.previous_revision}
                    <p>
                        Previous Revision: <span class="text-green-600"
                            >{change.previous_revision}</span
                        >
                    </p>
                {/if}
                <p>
                    New Revision: <span class="text-red-600">{change.new_revision}</span>
                </p>
            </div>
        </div>
    {/each}
</div>
