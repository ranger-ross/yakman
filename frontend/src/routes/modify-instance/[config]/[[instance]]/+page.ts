import { trpc } from "$lib/trpc/client";
import type { PageLoad } from "./$types";

export const load: PageLoad = async (event) => {
    const editMode = !!event.params.instance;
    let data = editMode ? await trpc(event).data.fetchInstanceData.query({
        configId: event.params.config,
        instance: event.params.instance!,
    }) : null;


    const labels = await trpc(event).labels.fetchLabels.query();

    let selectedLabels: { [labelName: string]: string } = {}; // <LabelName, Value>

    if (editMode) {
        const instances = await trpc(event).instances.fetchInstancesByConfigId.query(event.params.config);
        const instance = instances.find(i => i.instance == event.params.instance!);
        let selectedLabelsList = instance?.labels;

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