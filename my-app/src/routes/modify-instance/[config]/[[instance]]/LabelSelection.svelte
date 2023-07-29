<script lang="ts">
    import YakManSelect from "$lib/components/YakManSelect.svelte";
    import type { YakManLabelType } from "$lib/types/types";

    export let labels: YakManLabelType[] = [];
    export let selectedLabels: { [key: string]: string } | undefined =
        undefined;

    function onSelectChange(label: YakManLabelType, event: Event) {
        const value = (event.target! as HTMLSelectElement).value;

        if (selectedLabels) {
            selectedLabels[label.name] = value;
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
            >
                <option value="none" selected={true}> None </option>
                {#each label.options as option}
                    <option value={option}>{option}</option>
                {/each}
            </YakManSelect>
        {/each}
    </div>
</div>
