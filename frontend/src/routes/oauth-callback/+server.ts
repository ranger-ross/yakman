import type { RequestHandler } from './$types';
import { getYakManBaseApiUrl } from '$lib/trpc/helper';
import { json } from '@sveltejs/kit';

const BASE_URL = getYakManBaseApiUrl()

type OAuthExchangeResponse = {
    access_token: string,
    access_token_expire_timestamp: number,
    refresh_token: string | null,
}

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

    const { access_token, access_token_expire_timestamp, refresh_token } = await response.json() as OAuthExchangeResponse

    cookies.set('access_token', access_token, {
        httpOnly: true,
        path: '/',
        maxAge: access_token_expire_timestamp,
    })

    if (refresh_token) {
        cookies.set('refresh_token', refresh_token, {
            httpOnly: true,
            path: '/refresh-token',
            maxAge: Date.now() + (1000 * 60 * 60 * 24 * 356) // TODO: Dynamically set from metadata
        })

    }

    cookies.delete('oidc_nonce', { path: '/' })

    return json({
        data: "SUCCESS"
    });
}


