import type { YakManConfig, YakManConfigInstance, YakManProject } from "$lib/types/types";


type SvelteFetch = (input: RequestInfo | URL, init?: RequestInit | undefined) => Promise<Response>;

// TODO: FIX
const BASE_URL = "http://localhost:5173/api"
// const BASE_URL = "http://127.0.0.1:8000"

export async function fetchProjects(fetch: SvelteFetch): Promise<YakManProject[]> {
    const response = await fetch(`${BASE_URL}/v1/projects`);
    return await response.json();
}

export async function fetchConfigs(fetch: SvelteFetch, projectUuid?: string): Promise<YakManConfig[]> {
    const url = `${BASE_URL}/v1/configs` + (projectUuid ? `?project=${projectUuid}` : ``);
    const response = await fetch(url);
    return await response.json();
}

export async function fetchConfigMetadata(fetch: SvelteFetch, configName: string): Promise<YakManConfigInstance[]> {
    const response = await fetch(`${BASE_URL}/v1/configs/${configName}/instances`);
    return await response.json();
}

