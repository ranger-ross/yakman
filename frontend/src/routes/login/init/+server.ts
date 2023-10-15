import type { RequestHandler } from './$types';
import { getYakManBaseApiUrl } from '$lib/trpc/helper';
import { json } from '@sveltejs/kit';

const BASE_URL = getYakManBaseApiUrl()

type OAuthInitResponse = {
    redirect_uri: string,
    csrf_token: string,
    nonce: string,
}

export const POST: RequestHandler = async function ({ request, cookies, fetch }) {
    const { challenge } = await request.json();

    const response = await fetch(`${BASE_URL}/oauth2/init`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            'challenge': {
                'code_challenge': challenge.challenge,
                'code_challenge_method': challenge.codeChallengeMethod,
            }
        })
    });

    if (response.status != 200) {
        throw new Error(await response.text())
    }

    const { redirect_uri, nonce } = await response.json() as OAuthInitResponse;

    cookies.set('oidc_nonce', nonce, {
        httpOnly: true,
        path: '/'
    })

    return json({
        redirectUrl: redirect_uri
    });
}


