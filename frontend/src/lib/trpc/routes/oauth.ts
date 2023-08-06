import { z } from "zod";
import { t } from "../t";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";

const BASE_URL = getYakManBaseApiUrl();

export type GetUserRolesResponse = {
    global_roles: string[],
    roles: { [key: string]: string },
};

export const oauth = t.router({
    generateOauthRedirectUri: t.procedure
        .input(z.object({
            challenge: z.string(),
            challengeMethod: z.string(),
        }))
        .mutation(async ({ input }) => {
            const response = await fetch(`${BASE_URL}/oauth2/init`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    'challenge': {
                        'code_challenge': input.challenge,
                        'code_challenge_method': input.challengeMethod,
                    }
                })
            });

            if (response.status != 200) {
                throw new Error(await response.text())
            }
            return await response.text();
        }),
    fetchUserRoles: t.procedure
        .query(async ({ ctx }) => {
            const response = await fetch(`${BASE_URL}/oauth2/user-roles`, {
                headers: {
                    ...createYakManAuthHeaders(ctx.accessToken),
                }
            });
            if (response.status != 200) {
                throw new Error(await response.text())
            }
            return await response.json() as GetUserRolesResponse;
        })
})

