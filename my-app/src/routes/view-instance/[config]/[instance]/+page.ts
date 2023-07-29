import { trpc } from "$lib/trpc/client";
import type { YakManConfigInstance } from "$lib/types/types";
import type { PageLoad } from "./$types";

export const load: PageLoad = async (event) => {
    const { config, instance } = event.params

    const revisions = await trpc(event).revisions.fetchInstanceRevisions.query({
        configName: config,
        instance: instance,
    })

    const metadata = await trpc(event).instances.fetchConfigMetadata.query(config);

    let instanceMetadata: YakManConfigInstance | null = null;
    let data: { data: string; contentType: string; } | null = null;

    for (const inst of metadata) {
        if (inst.instance == instance) {
            instanceMetadata = inst;
        }
    }

    if (instanceMetadata) {
        data = await trpc(event).data.fetchRevisionData.query({
            configName: config,
            instance: instance,
            revision: instanceMetadata.current_revision
        });
    }


    return {
        data: data,
        instance: instanceMetadata,
        revisions: revisions
    };
};

