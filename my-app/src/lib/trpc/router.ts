// import type { Context } from '$lib/trpc/context';
import { z } from 'zod';
import { t } from './t';
import { createProject } from './routes/create-project';

export const router = t.router({
    createProject: createProject
});

export type Router = typeof router;
