import { trpc } from "$lib/trpc/client";
import type { PageLoad } from "../$types";

export const load: PageLoad = async (event) => {
    const t = trpc(event);

    return {
        users: await t.admin.fetchUsers.query(),
        apiKeys: await t.admin.fetchApiKeys.query(),
        projects: await t.projects.fetchProjects.query(),
    }
};