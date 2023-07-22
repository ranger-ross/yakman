import { t } from './t';
import { createProject } from './routes/create-project';
import { fetchProjects } from './routes/fetch-projects';
import { createContext } from './context';
import type { RequestEvent } from '@sveltejs/kit';
import { fetchConfigs } from './routes/fetch-configs';
import { fetchConfigMetadata } from './routes/fetch-config-metadata';

export const router = t.router({
    createProject: createProject,
    getProjects: fetchProjects,
    fetchConfigs: fetchConfigs,
    fetchConfigMetadata: fetchConfigMetadata,
});

export type Router = typeof router;

export async function createRouterCaller(event: RequestEvent) {
    return await router.createCaller(await createContext(event));
}