import { z } from "zod";
import { t } from "../t";
import type { YakManProject } from "$lib/types/types";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";
import { convertYakManErrorToTRPCError } from "$lib/utils/error-helpers";

const BASE_URL = getYakManBaseApiUrl();

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
        .input(z.string())
        .mutation(async ({ input, ctx }) => {
            const response = await fetch(`${BASE_URL}/v1/projects`, {
                method: 'PUT',
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    'project_name': input
                })
            });
            if (response.status != 200) {
                throw convertYakManErrorToTRPCError(await response.text(), response.status)
            }

            return {
                projectUuid: await response.text()
            }
        })
})


