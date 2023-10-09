import type { RequestHandler } from './$types';
import { getYakManBaseApiUrl } from '$lib/trpc/helper';
import { parse } from 'set-cookie-parser';
import { json } from '@sveltejs/kit';

const BASE_URL = getYakManBaseApiUrl()

export const POST: RequestHandler = async function ({ request, cookies, fetch }) {
    const { code, state, verifier } = await request.json();

    const nonceCookie = cookies.get('oidc_nonce')

    const response = await fetch(`${BASE_URL}/oauth2/exchange`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            code: code,
            state: state,
            verifier: verifier,
            nonce: nonceCookie
        })
    });

    if (response.status != 200) {
        throw new Error(await response.text())
    }

    for (const cookie of parse(response as any)) {
        if (cookie.name === 'refresh_token') {
            cookies.set(cookie.name, cookie.value, {
                httpOnly: cookie.httpOnly,
                path: '/refresh-token',
                maxAge: cookie.maxAge,
            });
        } else {
            cookies.set(cookie.name, cookie.value, {
                httpOnly: cookie.httpOnly,
                path: cookie.path,
                maxAge: cookie.maxAge,
            });
        }
    }

    cookies.delete('oidc_nonce')

    return json({
        data: await response.text()
    });
}


