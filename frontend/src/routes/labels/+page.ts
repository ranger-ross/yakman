import { trpc } from "$lib/trpc/client";
import type { PageLoad } from "./$types";

export const load: PageLoad = async (event) => {
    let labels = await trpc(event).labels.fetchLabels.query();

    return {
        labels: labels,
    };
}
