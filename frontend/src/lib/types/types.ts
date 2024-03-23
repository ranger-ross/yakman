import { z } from "zod";

export const YakManProjectSchema = z.object({
    id: z.string(),
    name: z.string(),
});

export type YakManProject = z.infer<typeof YakManProjectSchema>;

export const YakManProjectDetailsSchema = z.object({
    uuid: z.string(),
    name: z.string(),
    notification_settings: z.object({
        settings: z.object({
            Slack: z.object({
                webhook_url: z.string()
            }).optional()
        }),
        events: z.object({
            is_instance_updated_enabled: z.boolean(),
            is_instance_created_enabled: z.boolean(),
            is_revision_submitted_enabled: z.boolean(),
            is_revision_approved_enabled: z.boolean(),
            is_revision_reject_enabled: z.boolean()
        })
    }).optional()
});

export type YakManProjectDetails = z.infer<typeof YakManProjectDetailsSchema>;

export const YakManConfigSchema = z.object({
    id: z.string(),
    name: z.string(),
    project_id: z.string(),
    hidden: z.boolean(),
});

export type YakManConfig = z.infer<typeof YakManConfigSchema>;

export const YakManLabelTypeSchema = z.object({
    name: z.string(),
    description: z.string(),
    options: z.array(z.string()),
});

export type YakManLabelType = z.infer<typeof YakManLabelTypeSchema>;

export const YakManLabelSchema = z.object({
    label_type: z.string(),
    value: z.string(),
});

export type YakManLabel = z.infer<typeof YakManLabelSchema>;

export const ConfigInstanceEventSchema = z.object({
    timestamp_ms: z.number(),
    Created: z.object({
        new_revision: z.string(),
        created_by_uuid: z.string(),
    }).optional(),
    Updated: z.object({
        previous_revision: z.string(),
        new_revision: z.string(),
        created_by_uuid: z.string(),
    }).optional(),
    NewRevisionSubmitted: z.object({
        previous_revision: z.string(),
        new_revision: z.string(),
        submitted_by_uuid: z.string(),
    }).optional(),
    NewRevisionApproved: z.object({
        new_revision: z.string(),
        approver_by_uuid: z.string(),
    }).optional(),
    NewRevisionRejected: z.object({
        new_revision: z.string(),
        rejected_by_uuid: z.string(),
    }).optional()
});

export type ConfigInstanceEvent = z.infer<typeof ConfigInstanceEventSchema>;

export const YakManConfigInstanceSchema = z.object({
    config_id: z.string(),
    instance: z.string(),
    labels: z.array(YakManLabelSchema),
    current_revision: z.string(),
    pending_revision: z.string().nullable(),
    revisions: z.array(z.string()),
    changelog: z.array(ConfigInstanceEventSchema),
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

export const YakManApiKeySchema = z.object({
    id: z.string(),
    project_id: z.string(),
    role: z.string(),
    created_at: z.number().int(),
    created_by_uuid: z.string(),
});

export type YakManApiKey = z.infer<typeof YakManApiKeySchema>;

