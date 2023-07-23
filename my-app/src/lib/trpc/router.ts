import { t } from './t';
import { createProject } from './routes/create-project';
import { fetchProjects } from './routes/fetch-projects';
import { createContext } from './context';
import type { RequestEvent } from '@sveltejs/kit';
import { fetchConfigs } from './routes/fetch-configs';
import { fetchConfigMetadata } from './routes/fetch-config-metadata';
import { createLabel } from './routes/create-label';
import { fetchLabels } from './routes/fetch-labels';
import { createConfig } from './routes/create-config';
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