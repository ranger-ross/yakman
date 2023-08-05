<script lang="ts">
    import type { YakManInstanceRevision } from "$lib/types/types";

    export let currentRevision: string | null = null;
    export let pendingRevision: string | null = null;
    export let sortedRevisions: YakManInstanceRevision[] = [];

    function formatDate(ts: number): string {
        const date = new Date(ts);
        return date.toLocaleDateString() + " " + date.toLocaleTimeString();
    }

    // TODO: FIX
    // function onRevisionClicked(revision: any): any {
    //     throw new Error("Function not implemented.");
    // }
</script>

{#each sortedRevisions as revision}
    <div class="flex gap-2">
        <p>{formatDate(revision.timestamp_ms)} =></p>
        {#if revision.revision == currentRevision}
            <p class="text-yellow-400">{revision.revision}</p>
        {:else}
            <p
                class="text-blue-600 cursor-pointer"
                on:click={() => onRevisionClicked(revision)}
            >
                {revision.revision}
            </p>
        {/if}
        {#if revision.revision == pendingRevision}
            (pending)
        {/if}
    </div>
{/each}
