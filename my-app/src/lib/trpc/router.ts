import { t } from './t';
import { createProject, fetchProjects } from './routes/projects';
import { createContext } from './context';
import type { RequestEvent } from '@sveltejs/kit';
import { createConfig, fetchConfigs } from './routes/configs';
import { fetchConfigMetadata } from './routes/instances';
import { fetchLabels, createLabel } from './routes/labels';
import { generateOauthRedirectUri } from './routes/oauth';

export const router = t.router({
    createProject: createProject,
    fetchProjects: fetchProjects,
    fetchConfigs: fetchConfigs,
    fetchConfigMetadata: fetchConfigMetadata,
    fetchLabels: fetchLabels,
    createLabel: createLabel,
    createConfig: createConfig,
    generateOauthRedirectUri: generateOauthRedirectUri,
});

export type Router = typeof router;

export async function createRouterCaller(event: RequestEvent) {
    return await router.createCaller(await createContext(event));
}