import { trpc } from "$lib/trpc/client";
import type { LayoutServerLoad } from "./$types";
import { getYakManBaseApiUrl } from '$lib/trpc/helper';
import type { GetUserRolesResponse } from "$lib/trpc/routes/oauth";

const BASE_URL = getYakManBaseApiUrl()

export const load: LayoutServerLoad = async (event) => {
    const route = event.route.id;
    if (route === '/login') {
        return { userRoles: null }; // TODO: Handle this better (use null)
    }

    let userRoles: GetUserRolesResponse | null = null;
    let tokenRefreshNeeded = false;
    try {
        userRoles = await trpc(event).oauth.fetchUserRoles.query();

        if (
            userRoles.global_roles.length == 0 &&
            Object.keys(userRoles.roles).length == 0
        ) {
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

