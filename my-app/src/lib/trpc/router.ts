import { t } from './t';
import { createProject } from './routes/create-project';
import { fetchProjects } from './routes/fetch-projects';
import { createContext } from './context';
import type { RequestEvent } from '@sveltejs/kit';

export const router = t.router({
    createProject: createProject,
    getProjects: fetchProjects,
});

export type Router = typeof router;

export async function createRouterCaller(event: RequestEvent) {
    return await router.createCaller(await createContext(event));
}