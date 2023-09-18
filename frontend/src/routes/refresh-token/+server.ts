import type { RequestHandler } from './$types';
import { getYakManBaseApiUrl } from '$lib/trpc/helper';
import { parse } from 'set-cookie-parser';
import { error, json } from '@sveltejs/kit';

const BASE_URL = getYakManBaseApiUrl()

export const POST: RequestHandler = async function ({ cookies, fetch }) {
    const refreshToken = cookies.get('refresh_token');

    const response = await fetch(`${BASE_URL}/oauth2/refresh`, {
        method: 'POST',
        headers: !!refreshToken ? {
            'Cookie': `refresh_token=${refreshToken}`
        } : {}
    });

    if (response.status == 401) {
        throw error(401, {
            message: 'NAV_TO_LOGIN'
        })
    }

    if (response.status != 200) {
        throw error(401, {
            message: await response.text()
        })
    }

    for (const cookie of parse(response as any)) {
        cookies.set(cookie.name, cookie.value, {
            httpOnly: cookie.httpOnly,
            path: cookie.path,
            maxAge: cookie.maxAge,
        });
    }

    return json({
        data: await response.text()
    });
}
