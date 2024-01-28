import { t } from "../t";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";
import { z } from "zod";

const BASE_URL = getYakManBaseApiUrl();


export const auth = t.router({
    createResetPasswordLink: t.procedure
        .input(z.object({
            userUuid: z.string(),
        }))
        .mutation(async ({ ctx, input }) => {
            const response = await fetch(`${BASE_URL}/auth/create-reset-password-link`, {
                method: "POST",
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    user_uuid: input.userUuid
                })
            });
            if (response.status != 200) {
                throw new Error(await response.text())
            }
            return await response.json() as {
                id: string,
                user_uuid: string,
            };
        })
})

