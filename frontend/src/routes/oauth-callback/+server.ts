import type { RequestHandler } from './$types';
import { getYakManBaseApiUrl } from '$lib/trpc/helper';
import { parse } from 'set-cookie-parser';
import { json } from '@sveltejs/kit';

const BASE_URL = getYakManBaseApiUrl()

// TODO: clean up debug logs

export const POST: RequestHandler = async function ({ request, cookies, fetch }) {
    console.log('starting code exchange');
    const { code, state, verifier } = await request.json();

    console.log('sending request', `${BASE_URL}/oauth2/exchange`);


    const response = await fetch(`${BASE_URL}/oauth2/exchange`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            code: code,
            state: state,
            verifier: verifier,
        })
    });


    console.log('request recieved request', response.status);

    if (response.status != 200) {
        throw new Error(await response.text())
    }

    console.log('setting cookies');

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


    console.log('cookies set');

    return json({
        data: await response.text()
    });
}


