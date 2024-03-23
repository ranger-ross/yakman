import { trpc } from "$lib/trpc/client";
import type { PageLoad } from "./$types";


export const load: PageLoad = async (event) => {
    let id = event.url.searchParams.get('id') as string;
    let userId = event.url.searchParams.get('user_id') as string;

    const { valid } = await trpc(event).auth.validateResetPasswordLink.query({
        id,
        userId: userId
    });

    return {
        id,
        userId,
        isValidLink: valid
    }
};