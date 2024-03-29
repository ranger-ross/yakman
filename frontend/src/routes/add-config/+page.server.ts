import { createRouterCaller } from "$lib/trpc/router";
import type { PageServerLoad } from "./$types";

export const load: PageServerLoad = async (event) => {
    const trpc = await createRouterCaller(event);

    return {
        projects: await trpc.projects.fetchProjects()
    };
}

