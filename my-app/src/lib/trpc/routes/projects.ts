import { z } from "zod";
import { t } from "../t";
import type { YakManProject } from "$lib/types/types";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";

const BASE_URL = getYakManBaseApiUrl();

export const projects = t.router({
    fetchProjects: t.procedure
        .query(async ({ ctx }): Promise<YakManProject[]> => {
            const response = await fetch(`${BASE_URL}/v1/projects`, {
                headers: createYakManAuthHeaders(ctx.accessToken)
            });
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
                throw new Error(await response.text())
            }
        })
})


