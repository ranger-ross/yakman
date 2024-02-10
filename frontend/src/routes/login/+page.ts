import { trpc } from "$lib/trpc/client";
import type { PageLoad } from "./$types";

export const load: PageLoad = async (event) => {

    const settings = await trpc(event).lifecycle.fetchYakmanSettings.query();

    return {
        settings: settings,
        error: event.url.searchParams.get('error')
    };
}
