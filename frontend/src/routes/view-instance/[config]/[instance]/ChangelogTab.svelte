<script lang="ts">
    import type { ConfigInstanceChange } from "$lib/types/types";

    export let sortedChangelog: ConfigInstanceChange[] = [];

    function formatDate(ts: number): string {
        const date = new Date(ts);
        return date.toLocaleDateString() + " " + date.toLocaleTimeString();
    }
</script>

<div class="mt-2">
    {#each sortedChangelog as change}
        <div class="flex gap-2">
            <p>{formatDate(change.timestamp_ms)} =></p>
            {#if change.previous_revision}
                <p>Previous: {change.previous_revision} =></p>
            {/if}
            <p>New: {change.new_revision}</p>
        </div>
    {/each}
</div>
