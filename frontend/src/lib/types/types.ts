import { z } from "zod";

export const YakManProjectSchema = z.object({
    id: z.string(),
    name: z.string(),
});

export type YakManProject = z.infer<typeof YakManProjectSchema>;

export const YakManProjectDetailsSchema = z.object({
    id: z.string(),
    name: z.string(),
    notification_settings: z.object({
        settings: z.object({
            Slack: z.object({
                webhook_url: z.string()
            }).optional(),
            Discord: z.object({
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
    id: z.string(),
    name: z.string(),
    description: z.string(),
    options: z.array(z.string()),
});

export type YakManLabelType = z.infer<typeof YakManLabelTypeSchema>;

export const YakManLabelSchema = z.object({
    label_id: z.string(),
    name: z.string(),
    value: z.string(),
});

export type YakManLabel = z.infer<typeof YakManLabelSchema>;

export const ConfigInstanceEventSchema = z.object({
    timestamp_ms: z.number(),
    Created: z.object({
        new_revision: z.string(),
        created_by_user_id: z.string(),
    }).optional(),
    Updated: z.object({
        previous_revision: z.string(),
        new_revision: z.string(),
        created_by_user_id: z.string(),
    }).optional(),
    NewRevisionSubmitted: z.object({
        previous_revision: z.string(),
        new_revision: z.string(),
        submitted_by_user_id: z.string(),
    }).optional(),
    NewRevisionApproved: z.object({
        new_revision: z.string(),
        approver_by_user_id: z.string(),
    }).optional(),
    NewRevisionRejected: z.object({
        new_revision: z.string(),
        rejected_by_user_id: z.string(),
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
    reviewed_by_user_id: z.string().nullable(),
    review_timestamp_ms: z.number().int().nullable(),
    content_type: z.string(),
});

export type YakManInstanceRevision = z.infer<typeof YakManInstanceRevisionSchema>;

export const YakManRoleSchema = z.enum(['Viewer', 'Operator', 'Approver', 'Admin']);

export type YakManRole = z.infer<typeof YakManRoleSchema>;

export const YakManUserSchema = z.object({
    email: z.string(),
    id: z.string(),
    role: YakManRoleSchema,
});

export type YakManUser = z.infer<typeof YakManUserSchema>;

export const YakManApiKeySchema = z.object({
    id: z.string(),
    project_id: z.string(),
    role: z.string(),
    created_at: z.number().int(),
    created_by_user_id: z.string(),
});

export type YakManApiKey = z.infer<typeof YakManApiKeySchema>;

export const YakManTeamSchema = z.object({
    id: z.string(),
    name: z.string(),
});

export type YakManTeam = z.infer<typeof YakManTeamSchema>;

export const YakManTeamDetailsSchema = z.object({
    id: z.string(),
    name: z.string(),
    global_roles: z.array(YakManRoleSchema),
    roles: z.array(z.object({
        project_id: z.string(),
        role: YakManRoleSchema,
    })),
    member_user_ids: z.array(z.string()),
});

export type YakManTeamDetails = z.infer<typeof YakManTeamDetailsSchema>;
