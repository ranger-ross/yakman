import { trpc } from "$lib/trpc/client";
import type { PageData } from "./$types";

export const load: PageLoad = async (event) => {

    const config = await trpc(event).yakman.fetchYakmanConfig.query();

    return {
        config: config,
        error: event.url.searchParams.get('error')
    };
}
