import { t } from "../t";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";
import { z } from "zod";
import type { YakManUser } from "$lib/types/types";

const BASE_URL = getYakManBaseApiUrl();


export type GetUserInfoResponse = {
    profile_picture: string | null,
    global_roles: string[],
    roles: { [key: string]: string },
};

export const users = t.router({
    fetchUsers: t.procedure
        .query(async ({ ctx }): Promise<YakManUser[]> => {
            const response = await fetch(`${BASE_URL}/v1/users`, {
                headers: createYakManAuthHeaders(ctx.accessToken)
            });
            return await response.json();
        }),
    createUser: t.procedure
        .input(z.object({
            username: z.string(),
            role: z.string()
        }))
        .mutation(async ({ input, ctx }) => {
            const response = await fetch(`${BASE_URL}/v1/users`, {
                method: 'PUT',
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    'email': input.username,
                    'role': input.role
                })
            });
            if (response.status != 200) {
                throw new Error(await response.text())
            }
        }),
    fetchUserInfo: t.procedure
        .query(async ({ ctx }) => {
            const response = await fetch(`${BASE_URL}/v1/user-info`, {
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                }
            });
            if (response.status != 200) {
                throw new Error(await response.text())
            }
            return await response.json() as GetUserInfoResponse;
        })
})

