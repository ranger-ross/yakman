import { trpc } from "$lib/trpc/client";
import type { YakManConfigInstance } from "$lib/types/types";
import type { PageLoad } from "./$types";

export const load: PageLoad = async (event) => {
    const { config, instance } = event.params

    const revisions = await trpc(event).revisions.fetchInstanceRevisions.query({
        configId: config,
        instance: instance,
    })

    const metadata = await trpc(event).instances.fetchConfigMetadata.query(config);

    let instanceMetadata: YakManConfigInstance | null = null;
    let currentData: { data: string; contentType: string; } | null = null;
    let pendingData: { data: string; contentType: string; } | null = null;

    for (const inst of metadata) {
        if (inst.instance == instance) {
            instanceMetadata = inst;
        }
    }

    const pendingRevision = revisions.find(rev => rev.revision === instanceMetadata?.pending_revision)

    if (!pendingRevision) {
        console.error(`Failed to find revision ${instanceMetadata?.pending_revision}, this will likely prevent approve/reject/applying.`)
    }

    if (instanceMetadata) {
        currentData = await trpc(event).data.fetchRevisionData.query({
            configId: config,
            instance: instance,
            revision: instanceMetadata.current_revision
        });

        if (instanceMetadata.pending_revision) {
            pendingData = await trpc(event).data.fetchRevisionData.query({
                configId: config,
                instance: instance,
                revision: instanceMetadata.pending_revision
            });
        }
    }

    return {
        pendingRevision: pendingRevision,
        currentData: currentData,
        pendingData: pendingData,
    };
};
