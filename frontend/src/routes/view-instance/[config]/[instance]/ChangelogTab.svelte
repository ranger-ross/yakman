<script lang="ts">
    import type { ConfigInstanceEvent } from "$lib/types/types";
    import {
        getEventName as getChangelogEventName,
        getEventType as getChangelogEventType,
    } from "$lib/utils/changelog-utils";

    export let sortedChangelog: ConfigInstanceEvent[] = [];
    // Map<User UUID, Email>
    export let users: Map<string, string> = new Map();

    console.log(users);

    function formatDate(ts: number): string {
        const date = new Date(ts);
        return date.toLocaleDateString() + " " + date.toLocaleTimeString();
    }

    function getEmail(uuid: string): string {
        return users.get(uuid) ?? uuid; // Fallback to the user UUID
    }
</script>

<div class="mt-2">
    {#each sortedChangelog as change, index}
        {@const type = getChangelogEventType(change)}
        <div
            class={`grid grid-cols-2 my-2 p-1 px-2 rounded ${
                index % 2 == 0 ? "bg-gray-200" : "bg-gray-100"
            }`}
        >
            <div>
                <p class="font-bold">
                    {getChangelogEventName(type)}
                </p>

                <p class="text-sm text-gray-500">
                    {formatDate(change.timestamp_ms)}
                </p>
            </div>

            <div>
                {#if type === "CREATED"}
                    <p>
                        New Revision: <span class="text-green-600"
                            >{change.Created?.new_revision}</span
                        >
                    </p>
                {:else if type === "UPDATED"}
                    <p>
                        New Revision: <span class="text-green-600"
                            >{change.Updated?.new_revision}</span
                        >
                    </p>
                    <p>
                        Previous Revision: <span class="text-red-600"
                            >{change.Updated?.previous_revision}</span
                        >
                    </p>
                {:else if type === "SUBMITTED"}
                    <p>
                        Revision: <span class="text-blue-600"
                            >{change.NewRevisionSubmitted?.new_revision}</span
                        >
                    </p>
                    <p>
                        Approved by: {getEmail(
                            change.NewRevisionSubmitted?.submitted_by_uuid ??
                                "",
                        )}
                    </p>
                {:else if type === "APPROVED"}
                    <p>
                        Revision: <span class="text-blue-600"
                            >{change.NewRevisionApproved?.new_revision}</span
                        >
                    </p>
                    <p>
                        Approved by: {getEmail(
                            change.NewRevisionApproved?.approver_by_uuid ?? "",
                        )}
                    </p>
                {:else if type === "REJECTED"}
                    <p>
                        Revision: <span class="text-blue-600"
                            >{change.NewRevisionRejected?.new_revision}</span
                        >
                    </p>
                    <p>
                        Rejected by: {getEmail(
                            change.NewRevisionRejected?.rejected_by_uuid ?? "",
                        )}
                    </p>
                {/if}
            </div>
        </div>
    {/each}
</div>
