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
    reviewInstanceRevision: t.procedure
        .input(z.object({
            configName: z.string(),
            instance: z.string(),
            revision: z.string(),
            reviewResult: z.enum(["Approve", "ApproveAndApply", "Reject"])
        }))
        .mutation(async ({ input, ctx }) => {
            const response = await fetch(`${BASE_URL}/v1/configs/${input.configName}/instances/${input.instance}/revisions/${input.revision}/review/${input.reviewResult}`, {
                headers: createYakManAuthHeaders(ctx.accessToken),
                method: 'POST'
            });

            if (response.status != 200) {
                throw new Error(`failed to review revision: http-status [${response.status}]`);
            }
        }),
    applyInstanceRevision: t.procedure
        .input(z.object({
            configName: z.string(),
            instance: z.string(),
            revision: z.string(),
        }))
        .mutation(async ({ input, ctx }) => {
            const response = await fetch(`${BASE_URL}/v1/configs/${input.configName}/instances/${input.instance}/revisions/${input.revision}/apply`, {
                headers: createYakManAuthHeaders(ctx.accessToken),
                method: 'POST'
            });

            if (response.status != 200) {
                throw new Error(`failed to apply revision: http-status [${response.status}]`);
            }
        }),
});
