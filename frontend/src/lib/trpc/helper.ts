import { YAKMAN_API_URL } from '$env/static/private';

export function getYakManBaseApiUrl() {
    return YAKMAN_API_URL
}


export function createYakManAuthHeaders(token: string | undefined) {
    return {
        // TODO: Maybe convert this to a Bearer header?
        cookie: `access_token=${token}`
    }
}