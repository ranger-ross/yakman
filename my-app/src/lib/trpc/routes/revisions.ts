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
    fetchRevisionData: t.procedure
        .input(z.object({
            configName: z.string(),
            instance: z.string(),
            revision: z.string(),
        }))
        .query(async ({ input, ctx }) => {
            const response = await fetch(`${BASE_URL}/v1/configs/${input.configName}/instances/${input.instance}/revisions/${input.revision}/data`, {
                headers: createYakManAuthHeaders(ctx.accessToken)
            });
            const contentType = response.headers.get('content-type') ?? 'text/plain';
            const data = await response.text();
            return {
                contentType: contentType,
                data: data,
            };
        })
});
