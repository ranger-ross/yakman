import { trpc } from "$lib/trpc/client";
import type { PageLoad } from "./$types";

export const load: PageLoad = async (event) => {
    const editMode = !!event.params.instance;
    let data = editMode ? await trpc(event).data.fetchInstanceData.query({
        configName: event.params.config,
        instance: event.params.instance!,
    }) : null;


    const labels = await trpc(event).labels.fetchLabels.query();

    let selectedLabels: { [labelName: string]: string } = {}; // <LabelName, Value>

    if (editMode) {
        const meta = await trpc(event).instances.fetchConfigMetadata.query(event.params.config);
        const instanceMetadata = meta.find(i => i.instance == event.params.instance!);
        let selectedLabelsList = instanceMetadata?.labels;

        for (const label of labels) {
            if (selectedLabelsList) {
                const lbl = selectedLabelsList.find((x) => x.label_type == label.name);
                if (lbl) {
                    selectedLabels[label.name] = lbl.value;
                }
            }
        }
    }

    return {
        labels: labels,
        data: data,
        selectedLabels: selectedLabels
    };
}