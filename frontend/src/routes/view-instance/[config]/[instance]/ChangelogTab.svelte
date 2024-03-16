<script lang="ts">
    import type { ConfigInstanceEvent } from "$lib/types/types";

    export let sortedChangelog: ConfigInstanceEvent[] = [];

    function formatDate(ts: number): string {
        const date = new Date(ts);
        return date.toLocaleDateString() + " " + date.toLocaleTimeString();
    }

    type EventType =
        | "CREATED"
        | "UPDATED"
        | "SUBMITTED"
        | "APPROVED"
        | "REJECTED"
        | "UNKNOWN";

    function getEventType(change: ConfigInstanceEvent): EventType {
        if (change.Created) {
            return "CREATED";
        } else if (change.Updated) {
            return "UPDATED";
        } else if (change.NewRevisionSubmitted) {
            return "SUBMITTED";
        } else if (change.NewRevisionApproved) {
            return "APPROVED";
        } else if (change.NewRevisionRejected) {
            return "REJECTED";
        } else {
            return "UNKNOWN";
        }
    }

    function getEventName(type: EventType) {
        switch (type) {
            case "CREATED":
                return "Created";
            case "UPDATED":
                return "Revision Updated";
            case "SUBMITTED":
                return "Revision Submitted";
            case "APPROVED":
                return "Revision Approved";
            case "REJECTED":
                return "Revision Rejected";
            case "UNKNOWN":
                return "Unknown";
        }
    }
</script>

<div class="mt-2">
    {#each sortedChangelog as change, index}
        {@const type = getEventType(change)}
        <div
            class={`grid grid-cols-2 my-2 p-1 px-2 rounded ${
                index % 2 == 0 ? "bg-gray-200" : "bg-gray-100"
            }`}
        >
            <div>
                <p class="font-bold">
                    {getEventName(type)}
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
                        Approved by: {change.NewRevisionSubmitted
                            ?.submitted_by_uuid}
                    </p>
                {:else if type === "APPROVED"}
                    <p>
                        Revision: <span class="text-blue-600"
                            >{change.NewRevisionApproved?.new_revision}</span
                        >
                    </p>
                    <p>
                        Approved by: {change.NewRevisionApproved
                            ?.approver_by_uuid}
                    </p>
                {:else if type === "REJECTED"}
                    <p>
                        Revision: <span class="text-blue-600"
                            >{change.NewRevisionRejected?.new_revision}</span
                        >
                    </p>
                    <p>
                        Rejected by: {change.NewRevisionRejected
                            ?.rejected_by_uuid}
                    </p>
                {/if}
            </div>
        </div>
    {/each}
</div>
