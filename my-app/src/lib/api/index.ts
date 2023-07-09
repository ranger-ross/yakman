export type YakManProject = {
    uuid: string,
    name: string,
};

export type YakManConfig = {
    name: string,
    project_uuid: string,
    description: string,
    hidden: boolean,
};

export type YakManLabelType = {
    name: string,
    description: string,
    priority: number,
    options: string[],
};

export type YakManLabel = {
    label_type: string,
    value: string,
};

export type YakManConfigInstance = {
    config_name: string,
    instance: string,
    labels: YakManLabel[], // These should match the labels in the current revision
    current_revision: string,
    pending_revision: string | null,
    revisions: string[],
    changelog: ConfigInstanceChange[],
};

export type ConfigInstanceChange = {
    timestamp_ms: number,
    previous_revision: string | null,
    new_revision: string,
};




// TODO: FIX
const BASE_URL = "http://127.0.0.1:8000"

export async function fetchProjects(): Promise<YakManProject[]> {
    const response = await fetch(`${BASE_URL}/v1/projects`);
    return await response.json();
}

export async function fetchConfigs(projectUuid?: string): Promise<YakManConfig[]> {
    const url = `${BASE_URL}/v1/configs` + (projectUuid ? `?project_uuid=${projectUuid}` : ``);
    const response = await fetch(url);
    return await response.json();
}

export async function fetchConfigMetadata(configName: string): Promise<YakManConfigInstance[]> {
    const response = await fetch(`${BASE_URL}/v1/configs/${configName}/instances`);
    return await response.json();
}

