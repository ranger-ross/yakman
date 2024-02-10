import { t } from './t';
import { createContext } from './context';
import type { RequestEvent } from '@sveltejs/kit';
import { configs } from './routes/configs';
import { instances } from './routes/instances';
import { labels } from './routes/labels';
import { projects } from './routes/projects';
import { revisions } from './routes/revisions';
import { data } from './routes/data';
import { auth } from './routes/auth';
import { lifecycle } from './routes/lifecycle';
import { users } from './routes/users';
import { apiKeys } from './routes/api-keys';

export const router = t.router({
    configs: configs,
    projects: projects,
    labels: labels,
    instances: instances,
    revisions: revisions,
    data: data,
    users: users,
    apiKeys: apiKeys,
    auth: auth,
    lifecycle: lifecycle,
});

export type Router = typeof router;

export async function createRouterCaller(event: RequestEvent) {
    return await router.createCaller(await createContext(event));
}