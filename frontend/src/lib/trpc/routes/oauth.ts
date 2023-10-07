import { t } from "../t";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";

const BASE_URL = getYakManBaseApiUrl();

export type GetUserRolesResponse = {
    global_roles: string[],
    roles: { [key: string]: string },
};

export const oauth = t.router({
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

