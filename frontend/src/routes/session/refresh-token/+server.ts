import type { RequestHandler } from './$types';
import { getYakManBaseApiUrl } from '$lib/trpc/helper';
import { error, json } from '@sveltejs/kit';

const BASE_URL = getYakManBaseApiUrl()


type OAuthRefreshTokenResponse = {
    access_token: string,
    access_token_expire_timestamp: number,
}


export const POST: RequestHandler = async function ({ cookies, fetch }) {
    const refreshToken = cookies.get('refresh_token');

    const response = await fetch(`${BASE_URL}/auth/refresh`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            refresh_token: refreshToken as string
        })
    });

    if (response.status == 401) {
        error(401, {
            message: 'NAV_TO_LOGIN'
        });
    }

    if (response.status != 200) {
        error(401, {
            message: await response.text()
        });
    }

    const { access_token, access_token_expire_timestamp } = await response.json() as OAuthRefreshTokenResponse

    cookies.set('access_token', access_token, {
        httpOnly: true,
        path: '/',
        maxAge: access_token_expire_timestamp,
    });

    return json({
        data: "SUCCESS"
    });
}
