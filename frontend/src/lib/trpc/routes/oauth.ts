import { t } from "../t";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";

const BASE_URL = getYakManBaseApiUrl();

export type GetUserInfoResponse = {
    profile_picture: string | null,
    global_roles: string[],
    roles: { [key: string]: string },
};

export const oauth = t.router({
    fetchUserInfo: t.procedure
        .query(async ({ ctx }) => {
            const response = await fetch(`${BASE_URL}/oauth2/user-info`, {
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

