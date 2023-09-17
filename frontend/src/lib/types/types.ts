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
    applied_by_uuid: z.string()
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

export const YakManInstanceRevisionSchema = z.object({
    revision: z.string(), // Unique key
    data_key: z.string(), // Key to fetch data
    labels: z.array(YakManLabelSchema),
    timestamp_ms: z.number().int(),
    review_state: z.enum(['Pending', 'Approved', 'Rejected']),
    reviewed_by_uuid: z.string().nullable(),
    review_timestamp_ms: z.number().int().nullable(),
    content_type: z.string(),
});

export type YakManInstanceRevision = z.infer<typeof YakManInstanceRevisionSchema>;

export const YakManUserSchema = z.object({
    email: z.string(),
    uuid: z.string(),
});

export type YakManUser = z.infer<typeof YakManUserSchema>;

