import { trpc } from "$lib/trpc/client";
import type { PageLoad } from "./$types";

export const load: PageLoad = async (event) => {
    return {
        labels: await trpc(event).labels.fetchLabels.query()
    };
}