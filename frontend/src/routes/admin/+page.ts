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

    const users = await t.admin.fetchUsers.query();
    const apiKeys = await t.admin.fetchApiKeys.query();
    const projects = await t.projects.fetchProjects.query();

    const apiKeyTableRows = apiKeys.map(key => {
        return {
            id: key.id,
            projectName: projects.find(p => p.uuid === key.project_uuid)?.name,
            role: key.role,
            createdAt: new Date(key.created_at),
            createdBy: users.find(u => u.uuid === key.created_by_uuid)?.email
        } as ApiKeyTableRow
    });

    return {
        users: users,
        apiKeyTableRows: apiKeyTableRows,
        projects: projects,
    }
};