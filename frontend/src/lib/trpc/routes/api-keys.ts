import { z } from "zod";
import { t } from "../t";
import type { YakManApiKey } from "$lib/types/types";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";

const BASE_URL = getYakManBaseApiUrl();

export const apiKeys = t.router({
    fetchApiKeys: t.procedure
        .query(async ({ ctx }): Promise<YakManApiKey[]> => {
            const response = await fetch(`${BASE_URL}/v1/api-keys`, {
                headers: createYakManAuthHeaders(ctx.accessToken)
            });
            return await response.json();
        }),
    createApiKey: t.procedure
        .input(z.object({
            projectId: z.string(),
            role: z.string()
        }))
        .mutation(async ({ input, ctx }) => {
            const response = await fetch(`${BASE_URL}/v1/api-keys`, {
                method: 'PUT',
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    'project_id': input.projectId,
                    'role': input.role
                })
            });
            if (response.status != 200) {
                throw new Error(await response.text())
            }
            const json = await response.json();

            return json.api_key as string
        }),
    deleteApiKey: t.procedure
        .input(z.object({
            id: z.string(),
        }))
        .mutation(async ({ input, ctx }) => {
            const response = await fetch(`${BASE_URL}/v1/api-keys/${input.id}`, {
                method: 'DELETE',
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                    'Content-Type': 'application/json'
                }
            });
            if (response.status != 200) {
                throw new Error(await response.text())
            }
        }),
});


