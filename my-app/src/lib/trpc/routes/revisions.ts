import { t } from "../t";
import { z } from "zod";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";
import type { YakManInstanceRevision } from "$lib/types/types";

const BASE_URL = getYakManBaseApiUrl();

export const revisions = t.router({
    fetchInstanceRevisions: t.procedure
        .input(z.object({
            configName: z.string(),
            instance: z.string(),
        }))
        .query(async ({ input, ctx }): Promise<YakManInstanceRevision[]> => {
            const response = await fetch(`${BASE_URL}/v1/configs/${input.configName}/instances/${input.instance}/revisions`, {
                headers: createYakManAuthHeaders(ctx.accessToken)
            });

            return await response.json();
        }),
    approveInstanceRevision: t.procedure
        .input(z.object({
            configName: z.string(),
            instance: z.string(),
            revision: z.string(),
        }))
        .mutation(async ({ input, ctx }) => {
            const response = await fetch(`${BASE_URL}/v1/configs/${input.configName}/instances/${input.instance}/revisions/${input.revision}/approve`, {
                headers: createYakManAuthHeaders(ctx.accessToken),
                method: 'POST'
            });

            if (response.status != 200) {
                throw new Error(`failed to approve revision: http-status [${response.status}]`);
            }

        }),
    updateInstanceRevision: t.procedure
        .input(z.object({
            configName: z.string(),
            instance: z.string(),
            revision: z.string(),
        }))
        .mutation(async ({ input, ctx }) => {
            const response = await fetch(`${BASE_URL}/v1/configs/${input.configName}/instances/${input.instance}/revisions/${input.revision}/submit`, {
                headers: createYakManAuthHeaders(ctx.accessToken),
                method: 'PUT'
            });

            if (response.status != 200) {
                throw new Error(`failed to submit revision: http-status [${response.status}]`);
            }

        }),
});
