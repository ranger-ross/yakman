import { trpc } from "$lib/trpc/client";
import type { LayoutServerLoad } from "./$types";
import type { GetUserInfoResponse } from "$lib/trpc/routes/oauth";
import { hasRoles } from "$lib/utils/role-utils";

const noRoles: GetUserInfoResponse = {
    global_roles: [],
    roles: {},
    profile_picture: null
}

export const load: LayoutServerLoad = async (event) => {
    const route = event.route.id;
    if (route === '/login') {
        return { userRoles: noRoles };
    }

    let userRoles: GetUserInfoResponse = noRoles;
    let tokenRefreshNeeded = false;
    try {
        userRoles = await trpc(event).oauth.fetchUserInfo.query();

        if (!hasRoles(userRoles.global_roles, userRoles.roles)) {
            throw new Error("no user roles found");
        }
    } catch (e) {
        // TODO: check error (smarter handling)
        tokenRefreshNeeded = true;
    }

    return {
        userRoles: userRoles,
        tokenRefreshNeeded: tokenRefreshNeeded,
    };
};

