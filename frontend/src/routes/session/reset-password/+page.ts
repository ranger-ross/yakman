import { trpc } from "$lib/trpc/client";
import type { PageLoad } from "./$types";


export const load: PageLoad = async (event) => {
    let id = event.url.searchParams.get('id') as string;
    let userUuid = event.url.searchParams.get('user_uuid') as string;

    const { valid } = await trpc(event).auth.validateResetPasswordLink.query({
        id,
        userUuid
    });

    return {
        id,
        userUuid,
        isValidLink: valid
    }
};