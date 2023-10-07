import type { RequestHandler } from './$types';
import { getYakManBaseApiUrl } from '$lib/trpc/helper';
import { json } from '@sveltejs/kit';
import { copyCookiesFromResponse } from '$lib/utils/cookie-util';

const BASE_URL = getYakManBaseApiUrl()

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

    copyCookiesFromResponse(response, cookies);

    return json({
        redirectUrl: await response.text()
    });
}


