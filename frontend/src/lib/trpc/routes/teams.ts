import { t } from "../t";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";
import { z } from "zod";
import { YakManRoleSchema, type YakManTeam, type YakManTeamDetails } from "$lib/types/types";

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
            globalRole: YakManRoleSchema.optional(),
            roles: z.array(z.object({
                projectId: z.string(),
                role: YakManRoleSchema
            })),
            teamMembers: z.array(z.string())
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
                    global_roles: input.globalRole ? [input.globalRole] : [],
                    roles: input.roles.map(role => ({
                        project_id: role.projectId,
                        role: role.role,
                    })),
                    team_member_user_ids: input.teamMembers
                })
            });
            if (response.status != 200) {
                throw new Error(await response.text())
            }
        }),
    updateTeam: t.procedure
        .input(z.object({
            teamId: z.string(),
            name: z.string(),
            globalRole: YakManRoleSchema.optional(),
            roles: z.array(z.object({
                projectId: z.string(),
                role: YakManRoleSchema
            })),
            teamMembers: z.array(z.string())
        }))
        .mutation(async ({ input, ctx }) => {
            const response = await fetch(`${BASE_URL}/v1/teams/${input.teamId}`, {
                method: 'POST',
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    name: input.name,
                    global_roles: input.globalRole ? [input.globalRole] : [],
                    roles: input.roles.map(role => ({
                        project_id: role.projectId,
                        role: role.role,
                    })),
                    team_member_user_ids: input.teamMembers
                })
            });
            if (response.status != 200) {
                throw new Error(await response.text())
            }
        }),
    deleteTeam: t.procedure
        .input(z.object({
            teamId: z.string(),
        }))
        .mutation(async ({ input, ctx }) => {
            const response = await fetch(`${BASE_URL}/v1/teams/${input.teamId}`, {
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
})

