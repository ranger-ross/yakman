import { t } from "../t";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";
import { z } from "zod";

const BASE_URL = getYakManBaseApiUrl();


export type GetUserInfoResponse = {
    profile_picture: string | null,
    global_roles: string[],
    roles: { [key: string]: string },
};

export const auth = t.router({
    createResetPasswordLink: t.procedure
        .input(z.object({
            userId: z.string(),
        }))
        .mutation(async ({ ctx, input }) => {
            const response = await fetch(`${BASE_URL}/auth/create-reset-password-link`, {
                method: "POST",
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    user_id: input.userId
                })
            });
            if (response.status != 200) {
                throw new Error(await response.text())
            }
            return await response.json() as {
                id: string,
                user_id: string,
            };
        }),
    validateResetPasswordLink: t.procedure
        .input(z.object({
            id: z.string(),
            userId: z.string(),
        }))
        .query(async ({ ctx, input }) => {
            const response = await fetch(`${BASE_URL}/auth/validate-reset-password-link`, {
                method: "POST",
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    id: input.id,
                    user_id: input.userId
                })
            });
            if (response.status != 200) {
                throw new Error(await response.text())
            }
            return await response.json() as {
                valid: boolean,
            };
        }),
    resetPassword: t.procedure
        .input(z.object({
            id: z.string(),
            userId: z.string(),
            password: z.string(),
        }))
        .mutation(async ({ ctx, input }) => {
            const response = await fetch(`${BASE_URL}/auth/reset-password`, {
                method: "POST",
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    reset_link: {
                        id: input.id,
                        user_id: input.userId
                    },
                    password: input.password
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

