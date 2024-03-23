import { trpc } from "$lib/trpc/client";
import type { YakManProjectDetails } from "$lib/types/types";
import type { PageLoad } from "./$types";

export const load: PageLoad = async (event) => {
    const { id } = event.params;
    let project: YakManProjectDetails | null = null
    if (id) {
        project = await trpc(event).projects.fetchProject.query(id);
    }

    return {
        project: project
    };
};
