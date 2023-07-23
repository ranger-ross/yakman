import { z } from "zod";
import { t } from "../t";
import { getYakManBaseApiUrl } from "../helper";

const BASE_URL = getYakManBaseApiUrl();

export const generateOauthRedirectUri = t.procedure
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
    });
