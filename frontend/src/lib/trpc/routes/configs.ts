import { z } from "zod";
import { t } from "../t";
import type { YakManConfig } from "$lib/types/types";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";

const BASE_URL = getYakManBaseApiUrl();

export const configs = t.router({
    fetchConfigs: t.procedure
        .input(z.string().optional())
        .query(async ({ input, ctx }): Promise<YakManConfig[]> => {
            const response = await fetch(`${BASE_URL}/v1/configs` + (input ? `?project=${input}` : ``), {
                headers: createYakManAuthHeaders(ctx.accessToken)
            });
            return await response.json();
        }),

    createConfig: t.procedure
        .input(z.object({
            name: z.string(),
            projectUuid: z.string()
        }))
        .mutation(async ({ input, ctx }) => {
            const response = await fetch(`${BASE_URL}/v1/configs`, {
                method: 'PUT',
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    'config_name': input.name,
                    'project_uuid': input.projectUuid
                })
            });
            if (response.status != 200) {
                throw new Error(await response.text())
            }
            return {
                config: await response.text()
            }
        }),
    deleteConfig: t.procedure
        .input(z.object({
            name: z.string(),
            projectUuid: z.string()
        }))
        .mutation(async ({ input, ctx }) => {
            const response = await fetch(`${BASE_URL}/v1/configs`, {
                method: 'DELETE',
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    'config_name': input.name,
                    'project_uuid': input.projectUuid
                })
            });
            if (response.status != 200) {
                throw new Error(await response.text())
            }
        })
});
