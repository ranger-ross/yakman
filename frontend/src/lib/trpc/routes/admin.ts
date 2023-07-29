import { z } from "zod";
import { t } from "../t";
import type { YakManUser } from "$lib/types/types";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";

const BASE_URL = getYakManBaseApiUrl();

export const admin = t.router({
    fetchUsers: t.procedure
        .query(async ({ ctx }): Promise<YakManUser[]> => {
            const response = await fetch(`${BASE_URL}/admin/v1/users`, {
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
            const response = await fetch(`${BASE_URL}/admin/v1/users`, {
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
        })
});


