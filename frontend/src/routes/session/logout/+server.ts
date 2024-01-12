import type { RequestHandler } from './$types';
import { json } from '@sveltejs/kit';

export const POST: RequestHandler = async function ({ cookies, fetch }) {
    cookies.delete('refresh_token', {
        path: '/session'
    })
    cookies.delete('access_token', {
        path: '/'
    })

    return json({
        data: "SUCCESS"
    });
}
