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
