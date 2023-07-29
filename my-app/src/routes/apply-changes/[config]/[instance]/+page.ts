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

    let pendingRevision: string | null = null;
    let instanceMetadata: YakManConfigInstance | null = null;
    let currentData: { data: string; contentType: string; } | null = null;
    let pendingData: { data: string; contentType: string; } | null = null;

    for (const inst of metadata) {
        if (inst.instance == instance) {
            pendingRevision = inst.pending_revision;
            instanceMetadata = inst;
        }
    }

    if (instanceMetadata) {
        currentData = await trpc(event).data.fetchRevisionData.query({
            configName: config,
            instance: instance,
            revision: instanceMetadata.current_revision
        });

        if (instanceMetadata.pending_revision) {
            pendingData = await trpc(event).data.fetchRevisionData.query({
                configName: config,
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


// let mut revsions: Vec<ConfigInstanceRevision> = vec![];
// let mut pending_revision: Option<String> = None;
// let mut instance_metadata: Option<ConfigInstance> = None;
// let mut current_data: Option<(String, String)> = None;
// let mut pending_data: Option<(String, String)> = None;

// if let Ok(data) = api::fetch_instance_revisions(&config_name, &instance).await {
//     revsions = data;
// }

// let metadata = api::fetch_config_metadata(&config_name).await; // TODO: add a instance query param to avoid over fetching data

// for inst in metadata {
//     if inst.instance == instance {
//         pending_revision = inst.pending_revision.clone();
//         instance_metadata = Some(inst);
//     }
// }

// if let Some(instance_metadata) = instance_metadata {
//     let current_rev = instance_metadata.current_revision;
//     let pending_rev = instance_metadata.pending_revision.unwrap();

//     current_data = api::fetch_revision_data(&config_name, &instance, &current_rev)
//         .await
//         .ok();

//     pending_data = api::fetch_revision_data(&config_name, &instance, &pending_rev)
//         .await
//         .ok();
// }

// ApplyConfigPageData {
//     revisions: revsions,
//     pending_revision: pending_revision,
//     pending_revision_data: pending_data,
//     current_revision_data: current_data,
// }