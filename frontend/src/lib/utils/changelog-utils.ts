import type { ConfigInstanceEvent } from "$lib/types/types";

export type ChangelogEventType =
    | "CREATED"
    | "UPDATED"
    | "SUBMITTED"
    | "APPROVED"
    | "REJECTED"
    | "UNKNOWN";

export function getEventType(change: ConfigInstanceEvent): ChangelogEventType {
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

export function getEventName(type: ChangelogEventType) {
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