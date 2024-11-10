import { trpc } from "$lib/trpc/client";
import type { PageLoad } from "./$types";

export const load: PageLoad = async (event) => {
    const label = event.params.label;
    let labels = await trpc(event).labels.fetchLabels.query();

    return {
        labels: labels,
        label
    };
}
