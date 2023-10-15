import { env } from '$env/dynamic/private';

export function getYakManBaseApiUrl() {
    return env.YAKMAN_API_URL
}


export function createYakManAuthHeaders(token: string | undefined) {
    return {
        Authorization: `Bearer ${token}`
    }
}