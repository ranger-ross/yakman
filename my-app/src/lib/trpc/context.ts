import type { RequestEvent } from '@sveltejs/kit';
import type { inferAsyncReturnType } from '@trpc/server';

export async function createContext(event: RequestEvent) {
    const accessToken = event.cookies.get('access_token');
    return {
        accessToken: accessToken
    };
}

export type Context = inferAsyncReturnType<typeof createContext>;
