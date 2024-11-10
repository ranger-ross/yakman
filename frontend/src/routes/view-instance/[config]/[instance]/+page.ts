import { trpc } from "$lib/trpc/client";
import type { YakManConfigInstance } from "$lib/types/types";
import { getEventType } from "$lib/utils/changelog-utils";
import type { PageLoad } from "./$types";

export const load: PageLoad = async (event) => {
    const { config, instance: instanceId } = event.params

    const revisions = await trpc(event).revisions.fetchInstanceRevisions.query({
        configId: config,
        instance: instanceId,
    })

    const instances = await trpc(event).instances.fetchInstancesByConfigId.query(config);
    const labels = await trpc(event).labels.fetchLabels.query();

    let instance: YakManConfigInstance | null = null;
    let data: { data: string; contentType: string; } | null = null;
    let users: Map<string, string> = new Map()

    for (const inst of instances) {
        if (inst.instance == instanceId) {
            instance = inst;
        }
    }

    if (instance) {
        data = await trpc(event).data.fetchRevisionData.query({
            configId: config,
            instance: instanceId,
            revision: instance.current_revision
        });

        // Fetch the usernames of the users that made changes 
        // so they can be displayed in the changelog section

        try {
            const allUsers = await trpc(event).users.fetchUsers.query()

            const userIds = new Set(instance.changelog
                .map(change => {
                    const type = getEventType(change);
                    switch (type) {
                        case "CREATED":
                            return change.Created?.created_by_user_id;
                        case "UPDATED":
                            return change.Updated?.created_by_user_id;
                        case "SUBMITTED":
                            return change.NewRevisionSubmitted?.submitted_by_user_id;
                        case "APPROVED":
                            return change.NewRevisionApproved?.approver_by_user_id;
                        case "REJECTED":
                            return change.NewRevisionRejected?.rejected_by_user_id;;
                        case "UNKNOWN":
                            return null;
                    }
                })
                .filter(userId => !!userId))


            for (const user of allUsers) {
                if (userIds.has(user.id)) {
                    users.set(user.id, user.email)
                }
            }
        } catch {
            console.warn('Failed to load users, this probably means the users is not an admin')
        }
    }


    return {
        data: data,
        labels: labels,
        instance: instance,
        revisions: revisions,
        tab: getTab(event.url.searchParams),
        users: users
    };
};

function getTab(searchParams: URLSearchParams): "Changelog" | "Revisions" | null {
    let tab = searchParams.get('tab');
    if (['Changelog', 'Revisions'].includes(tab as string))
        return tab as "Changelog" | "Revisions"
    return null;
}
