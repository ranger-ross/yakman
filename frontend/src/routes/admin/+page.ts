import { trpc } from "$lib/trpc/client";
import type { PageLoad } from "../$types";

type ApiKeyTableRow = {
    id: string,
    projectName: string,
    role: string,
    createdAt: Date,
    createdBy: string
}

export const load: PageLoad = async (event) => {
    const t = trpc(event);

    const users = await t.users.fetchUsers.query();
    const apiKeys = await t.apiKeys.fetchApiKeys.query();
    const projects = await t.projects.fetchProjects.query();
    const settings = await t.lifecycle.fetchYakmanSettings.query();

    const apiKeyTableRows = apiKeys.map(key => {
        return {
            id: key.id,
            projectName: projects.find(p => p.id === key.project_id)?.name,
            role: key.role,
            createdAt: new Date(key.created_at),
            createdBy: users.find(u => u.id === key.created_by_user_id)?.email
        } as ApiKeyTableRow
    });

    return {
        users: users,
        apiKeyTableRows: apiKeyTableRows,
        projects: projects,
        tab: getTab(event.url.searchParams),
        settings
    }
};

function getTab(searchParams: URLSearchParams): "Users" | "Api Keys" | null {
    let tab = searchParams.get('tab');
    if (["Users", "Api Keys"].includes(tab as string))
        return tab as "Users" | "Api Keys"
    return null;
}