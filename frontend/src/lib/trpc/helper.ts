import { env } from '$env/dynamic/private';

export function getYakManBaseApiUrl() {
    return env.YAKMAN_API_URL
}


export function createYakManAuthHeaders(token: string | undefined) {
    return {
        // TODO: Maybe convert this to a Bearer header?
        cookie: `access_token=${token}`
    }
}