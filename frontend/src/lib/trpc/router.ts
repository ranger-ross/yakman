import { t } from './t';
import { createContext } from './context';
import type { RequestEvent } from '@sveltejs/kit';
import { configs } from './routes/configs';
import { instances } from './routes/instances';
import { labels } from './routes/labels';
import { oauth } from './routes/oauth';
import { projects } from './routes/projects';
import { revisions } from './routes/revisions';
import { data } from './routes/data';
import { admin } from './routes/admin';

export const router = t.router({
    oauth: oauth,
    configs: configs,
    projects: projects,
    labels: labels,
    instances: instances,
    revisions: revisions,
    data: data,
    admin: admin,
});

export type Router = typeof router;

export async function createRouterCaller(event: RequestEvent) {
    return await router.createCaller(await createContext(event));
}