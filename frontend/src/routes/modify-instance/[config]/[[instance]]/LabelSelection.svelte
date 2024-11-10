<script lang="ts">
    import YakManSelect from "$lib/components/YakManSelect.svelte";
    import type { YakManLabelType } from "$lib/types/types";

    export let labels: YakManLabelType[] = [];
    export let selectedLabels: { [key: string]: string };

    function onSelectChange(label: YakManLabelType, event: Event) {
        const value = (event.target! as HTMLSelectElement).value;
        const newValue = value === "" ? null : value;

        if (selectedLabels) {
            if (newValue) {
                selectedLabels[label.id] = newValue;
            } else {
                delete selectedLabels[label.id];
                selectedLabels = selectedLabels; // force Svelte to re-render
            }
            console.log("updated", selectedLabels);
        }
    }
</script>

<div class="my-4">
    <h1 class="text-lg font-bold mb-1">Labels</h1>
    <div class="flex flex-col gap-2">
        {#each labels as label}
            <YakManSelect
                label={label.name}
                on:change={(e) => onSelectChange(label, e)}
                value={selectedLabels?.[label.id] ?? undefined}
            >
                <option value={undefined}> None </option>
                {#each label.options as option}
                    <option value={option}>
                        {option}
                    </option>
                {/each}
            </YakManSelect>
        {/each}
    </div>
</div>
