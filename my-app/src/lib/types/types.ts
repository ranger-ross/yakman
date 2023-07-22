import { z } from "zod";

export const YakManProjectSchema = z.object({
    uuid: z.string(),
    name: z.string(),
});

export type YakManProject = z.infer<typeof YakManProjectSchema>;

export const YakManConfigSchema = z.object({
    name: z.string(),
    project_uuid: z.string(),
    description: z.string(),
    hidden: z.boolean(),
});

export type YakManConfig = z.infer<typeof YakManConfigSchema>;

export const YakManLabelTypeSchema = z.object({
    name: z.string(),
    description: z.string(),
    priority: z.number().int(),
    options: z.array(z.string()),
});

export type YakManLabelType = z.infer<typeof YakManLabelTypeSchema>;

export const YakManLabelSchema = z.object({
    label_type: z.string(),
    value: z.string(),
});

export type YakManLabel = z.infer<typeof YakManLabelSchema>;

export const ConfigInstanceChangeSchema = z.object({
    timestamp_ms: z.number(),
    previous_revision: z.string().nullable(),
    new_revision: z.string(),
});

export type ConfigInstanceChange = z.infer<typeof ConfigInstanceChangeSchema>;

export const YakManConfigInstanceSchema = z.object({
    config_name: z.string(),
    instance: z.string(),
    labels: z.array(YakManLabelSchema),
    current_revision: z.string(),
    pending_revision: z.string().nullable(),
    revisions: z.array(z.string()),
    changelog: z.array(ConfigInstanceChangeSchema),
});

export type YakManConfigInstance = z.infer<typeof YakManConfigInstanceSchema>;


