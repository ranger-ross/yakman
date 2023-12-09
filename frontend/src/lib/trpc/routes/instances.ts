import { z } from "zod";
import { t } from "../t";
import type { YakManConfigInstance } from "$lib/types/types";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";

const BASE_URL = getYakManBaseApiUrl();

type InstanceResponse = {
    instance: string
}

export const instances = t.router({
    fetchConfigMetadata: t.procedure
        .input(z.string())
        .query(async ({ input, ctx }): Promise<YakManConfigInstance[]> => {
            const response = await fetch(`${BASE_URL}/v1/configs/${input}/instances`, {
                headers: createYakManAuthHeaders(ctx.accessToken)
            });
            return await response.json();
        }),
    createConfigInstance: t.procedure
        .input(z.object({
            configName: z.string(),
            data: z.string(),
            contentType: z.string(),
            labels: z.record(z.string(), z.string())
        }))
        .mutation(async ({ input, ctx }) => {
            let query = '';
            if (input.labels != null && Object.keys(input.labels).length > 0) {
                query = '?' + new URLSearchParams(input.labels);
            }

            const response = await fetch(`${BASE_URL}/v1/configs/${input.configName}/instances${query}`, {
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                    'content-type': input.contentType ?? 'text/plain'
                },
                method: 'PUT',
                body: input.data
            });

            if (response.status != 200) {
                throw new Error(`failed to create config instance, http-status: [${response.status}]`);
            }

            const data = await response.json() as InstanceResponse;

            return {
                instance: data.instance
            };
        }),
    updateConfigInstance: t.procedure
        .input(z.object({
            configName: z.string(),
            instance: z.string(),
            data: z.string(),
            contentType: z.string(),
            labels: z.record(z.string(), z.string())
        }))
        .mutation(async ({ input, ctx }) => {
            let query = '';
            if (input.labels != null && Object.keys(input.labels).length > 0) {
                query = '?' + new URLSearchParams(input.labels);
            }

            const response = await fetch(`${BASE_URL}/v1/configs/${input.configName}/instances/${input.instance}${query}`, {
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                    'content-type': input.contentType ?? 'text/plain'
                },
                method: 'POST',
                body: input.data
            });

            if (response.status != 200) {
                throw new Error(`failed to update config instance, http-status: [${response.status}]`);
            }
        })
});

