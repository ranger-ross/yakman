import { z } from "zod";
import { t } from "../t";
import type { YakManProject, YakManProjectDetails } from "$lib/types/types";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";
import { convertYakManErrorToTRPCError } from "$lib/utils/error-helpers";

const BASE_URL = getYakManBaseApiUrl();

const ModifyProjectPayloadSchema = z.object({
    name: z.string(),
    slack: z.object({
        webhookUrl: z.string()
    }).optional(),
    notificationEvents: z.object({
        isInstanceCreateEventEnabled: z.boolean(),
        isInstanceUpdateEventEnabled: z.boolean(),
        isRevisionSubmittedEventEnabled: z.boolean(),
        isRevisionApprovedEventEnabled: z.boolean(),
        isRevisionRejectedEventEnabled: z.boolean(),
    }).optional()
});

export type ModifyProjectPayload = z.infer<typeof ModifyProjectPayloadSchema>;

export const projects = t.router({
    fetchProjects: t.procedure
        .query(async ({ ctx }): Promise<YakManProject[]> => {
            const response = await fetch(`${BASE_URL}/v1/projects`, {
                headers: createYakManAuthHeaders(ctx.accessToken)
            });
            if (response.status != 200) {
                throw convertYakManErrorToTRPCError(await response.text(), response.status)
            }
            return await response.json();
        }),
    fetchProject: t.procedure
        .input(z.string())
        .query(async ({ input, ctx }): Promise<YakManProjectDetails> => {
            const response = await fetch(`${BASE_URL}/v1/projects/${input}`, {
                headers: createYakManAuthHeaders(ctx.accessToken)
            });
            if (response.status != 200) {
                throw convertYakManErrorToTRPCError(await response.text(), response.status)
            }
            return await response.json();
        }),
    createProject: t.procedure
        .input(ModifyProjectPayloadSchema)
        .mutation(async ({ input, ctx }) => {
            const body = createModifyProjectPayload(input)
            const response = await fetch(`${BASE_URL}/v1/projects`, {
                method: 'PUT',
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(body)
            });
            if (response.status != 200) {
                throw convertYakManErrorToTRPCError(await response.text(), response.status)
            }

            return {
                projectId: await response.text()
            }
        }),
    updateProject: t.procedure
        .input(z.object({
            projectId: z.string(),
            payload: ModifyProjectPayloadSchema
        }))
        .mutation(async ({ input, ctx }) => {
            const body = createModifyProjectPayload(input.payload)
            console.log(body)
            const response = await fetch(`${BASE_URL}/v1/projects/${input.projectId}`, {
                method: 'POST',
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(body)
            });
            if (response.status != 200) {
                throw convertYakManErrorToTRPCError(await response.text(), response.status)
            }

            return {
                projectId: await response.text()
            }
        }),
    deleteProject: t.procedure
        .input(z.string())
        .mutation(async ({ input, ctx }): Promise<undefined> => {
            const response = await fetch(`${BASE_URL}/v1/projects/${input}`, {
                method: 'DELETE',
                headers: createYakManAuthHeaders(ctx.accessToken)
            });
            if (response.status != 200) {
                throw convertYakManErrorToTRPCError(await response.text(), response.status)
            }
        }),
})

function createModifyProjectPayload(request: ModifyProjectPayload): any {
    let body: any = {
        'project_name': request.name
    }

    if (request.notificationEvents && request.slack) {
        body.notification_settings = {
            notification_type: {
                Slack: {
                    webhook_url: request.slack.webhookUrl
                }
            },
            is_instance_updated_enabled: request.notificationEvents.isInstanceUpdateEventEnabled,
            is_instance_created_enabled: request.notificationEvents.isInstanceCreateEventEnabled,
            is_revision_submitted_enabled: request.notificationEvents.isRevisionSubmittedEventEnabled,
            is_revision_approved_enabled: request.notificationEvents.isRevisionApprovedEventEnabled,
            is_revision_reject_enabled: request.notificationEvents.isRevisionRejectedEventEnabled,
        };
    }
    return body;
}
