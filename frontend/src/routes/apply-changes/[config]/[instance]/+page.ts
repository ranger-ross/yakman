import { trpc } from "$lib/trpc/client";
import type { YakManConfigInstance } from "$lib/types/types";
import type { PageLoad } from "./$types";

export const load: PageLoad = async (event) => {
    const { config, instance: instanceId } = event.params

    const revisions = await trpc(event).revisions.fetchInstanceRevisions.query({
        configId: config,
        instance: instanceId,
    })

    const instances = await trpc(event).instances.fetchInstancesByConfigId.query(config);

    let instance: YakManConfigInstance | null = null;
    let currentData: { data: string; contentType: string; } | null = null;
    let pendingData: { data: string; contentType: string; } | null = null;

    for (const inst of instances) {
        if (inst.instance == instanceId) {
            instance = inst;
        }
    }

    const pendingRevision = revisions.find(rev => rev.revision === instance?.pending_revision)

    if (!pendingRevision) {
        console.error(`Failed to find revision ${instance?.pending_revision}, this will likely prevent approve/reject/applying.`)
    }

    if (instance) {
        currentData = await trpc(event).data.fetchRevisionData.query({
            configId: config,
            instance: instanceId,
            revision: instance.current_revision
        });

        if (instance.pending_revision) {
            pendingData = await trpc(event).data.fetchRevisionData.query({
                configId: config,
                instance: instanceId,
                revision: instance.pending_revision
            });
        }
    }

    return {
        pendingRevision: pendingRevision,
        currentData: currentData,
        pendingData: pendingData,
    };
};
