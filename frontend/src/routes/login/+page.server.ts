import { getYakManBaseApiUrl } from "$lib/trpc/helper";
import { redirect, type Actions } from "@sveltejs/kit";

const BASE_URL = getYakManBaseApiUrl()

type LoginResponse = {
    access_token: string,
    access_token_expire_timestamp: number,
    refresh_token: string | null,
}

export const actions = {
    default: async ({ request, cookies }) => {
        const data = await request.formData();
        const username = data.get('username');
        const password = data.get('password');

        const response = await fetch(`${BASE_URL}/auth/login`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/x-www-form-urlencoded'
            },
            body: `username=${username}&password=${password}`
        });

        if (response.status == 401) {
            const error = encodeURI('Username or password was not correct')
            redirect(303, `/login?error=${error}`)
            return;
        }

        if (response.status != 200) {
            const error = encodeURI('An error occured')
            redirect(303, `/login?error=${error}`)
            return;
        }

        const { access_token, access_token_expire_timestamp, refresh_token }: LoginResponse = await response.json();

        // todo: maybe refactor this to a utils class since they are also used in the oauth and refresh token flows
        cookies.set('access_token', access_token, {
            httpOnly: true,
            path: '/',
            maxAge: access_token_expire_timestamp,
        })

        if (refresh_token) {
            cookies.set('refresh_token', refresh_token, {
                httpOnly: true,
                path: '/session',
                maxAge: Date.now() + (1000 * 60 * 60 * 24 * 356) // TODO: Dynamically set from metadata
            })
        }

        redirect(303, '/')
    }
} satisfies Actions;