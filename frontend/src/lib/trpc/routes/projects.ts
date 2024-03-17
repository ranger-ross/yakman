import { z } from "zod";
import { t } from "../t";
import type { YakManProject } from "$lib/types/types";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";
import { convertYakManErrorToTRPCError } from "$lib/utils/error-helpers";

const BASE_URL = getYakManBaseApiUrl();

const CreateProjectPayloadSchema = z.object({
    name: z.string(),
    slack: z.object({
        webhookUrl: z.string()
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
        .query(async ({ input, ctx }): Promise<YakManProject> => {
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

            if (input.slack) {
                body.notification_settings = {
                    Slack: {
                        webhook_url: input.slack.webhookUrl
                    }
                };
            }
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


