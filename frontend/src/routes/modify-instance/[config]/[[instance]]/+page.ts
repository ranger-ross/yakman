import { trpc } from "$lib/trpc/client";
import type { PageLoad } from "./$types";

export const load: PageLoad = async (event) => {
    const editMode = !!event.params.instance;
    let data = editMode ? await trpc(event).data.fetchInstanceData.query({
        configName: event.params.config,
        instance: event.params.instance!,
    }) : null;


    return {
        labels: await trpc(event).labels.fetchLabels.query(),
        data: data
    };
}