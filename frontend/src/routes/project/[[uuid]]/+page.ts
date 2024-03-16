import { trpc } from "$lib/trpc/client";
import type { YakManConfigInstance, YakManProject } from "$lib/types/types";
import type { PageLoad } from "./$types";

export const load: PageLoad = async (event) => {
    const { uuid } = event.params
    let project: YakManProject | null = null
    if (uuid) {
        project = await trpc(event).projects.fetchProject.query(uuid);
    }

    return {
        project: project
    };
};
