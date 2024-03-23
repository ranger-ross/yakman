import { t } from "../t";
import { z } from "zod";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";

const BASE_URL = getYakManBaseApiUrl();

export const data = t.router({
    fetchInstanceData: t.procedure
        .input(z.object({
            configId: z.string(),
            instance: z.string(),
        }))
        .query(async ({ input, ctx }) => {
            const response = await fetch(`${BASE_URL}/v1/configs/${input.configId}/instances/${input.instance}/data`, {
                headers: createYakManAuthHeaders(ctx.accessToken)
            });
            const contentType = response.headers.get('content-type') ?? 'text/plain';
            const data = await response.text();
            return {
                contentType: contentType,
                data: data,
            };
        }),
    fetchRevisionData: t.procedure
        .input(z.object({
            configId: z.string(),
            instance: z.string(),
            revision: z.string(),
        }))
        .query(async ({ input, ctx }) => {
            const response = await fetch(`${BASE_URL}/v1/configs/${input.configId}/instances/${input.instance}/revisions/${input.revision}/data`, {
                headers: createYakManAuthHeaders(ctx.accessToken)
            });
            const contentType = response.headers.get('content-type') ?? 'text/plain';
            const data = await response.text();
            return {
                contentType: contentType,
                data: data,
            };
        }),
});
