import { z } from "zod";
import { t } from "../t";
import type { YakManProject, YakManProjectDetails } from "$lib/types/types";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";
import { convertYakManErrorToTRPCError } from "$lib/utils/error-helpers";

const BASE_URL = getYakManBaseApiUrl();

const CreateProjectPayloadSchema = z.object({
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

export type CreateProjectPayload = z.infer<typeof CreateProjectPayloadSchema>;

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
        .input(CreateProjectPayloadSchema)
        .mutation(async ({ input, ctx }) => {
            let body: any = {
                'project_name': input.name
            }

            if (input.notificationEvents && input.slack) {
                body.notification_settings = {
                    notification_type: {
                        Slack: {
                            webhook_url: input.slack.webhookUrl
                        }
                    },
                    is_instance_updated_enabled: input.notificationEvents.isInstanceUpdateEventEnabled,
                    is_instance_created_enabled: input.notificationEvents.isInstanceCreateEventEnabled,
                    is_revision_submitted_enabled: input.notificationEvents.isRevisionSubmittedEventEnabled,
                    is_revision_approved_enabled: input.notificationEvents.isRevisionApprovedEventEnabled,
                    is_revision_reject_enabled: input.notificationEvents.isRevisionRejectedEventEnabled,
                };
            }
            console.log(body)
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
                projectUuid: await response.text()
            }
        })
})


