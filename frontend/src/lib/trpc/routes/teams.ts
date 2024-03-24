import { t } from "../t";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";
import { z } from "zod";
import type { YakManTeam, YakManTeamDetails } from "$lib/types/types";

const BASE_URL = getYakManBaseApiUrl();

export const teams = t.router({
    fetchTeams: t.procedure
        .query(async ({ ctx }): Promise<YakManTeam[]> => {
            const response = await fetch(`${BASE_URL}/v1/teams`, {
                headers: createYakManAuthHeaders(ctx.accessToken)
            });
            return await response.json();
        }),
    fetchTeamById: t.procedure
        .input(z.string())
        .query(async ({ input, ctx }): Promise<YakManTeamDetails> => {
            const response = await fetch(`${BASE_URL}/v1/teams/${input}`, {
                headers: createYakManAuthHeaders(ctx.accessToken)
            });
            return await response.json();
        }),
    createTeam: t.procedure
        .input(z.object({
            name: z.string(),
        }))
        .mutation(async ({ input, ctx }) => {
            const response = await fetch(`${BASE_URL}/v1/teams`, {
                method: 'PUT',
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    name: input.name,
                    global_roles: [], // TODO: add roles
                    roles: [], // TODO: add roles
                })
            });
            if (response.status != 200) {
                throw new Error(await response.text())
            }
        }),
})

