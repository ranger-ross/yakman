<script lang="ts">
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import { page } from "$app/stores";
    import { trpc } from "$lib/trpc/client";
    import type { YakManLabelType } from "$lib/types/types";
    import { goto } from "$app/navigation";

    let name = "";
    let prioity = "";
    let description = "";
    let options = "";

    async function onCreateLabel() {
        try {
            const label: YakManLabelType = {
                name: name,
                description: description,
                priority: parseInt(prioity),
                options: options.split(",").filter((o) => !!o || o.length == 0),
            };

            await trpc($page).labels.createLabel.mutate(label);
            goto("/");
        } catch (e) {
            console.error("Error creating config:", e);
        }
    }
</script>

<div class="container mx-auto">
    <YakManCard>
        <h1 class="text-lg font-bold mb-4">{"Add Label"}</h1>
        <div class="mb-3">
            <YakManInput
                label="Name"
                bind:value={name}
                placeholder="my-label-name"
                mask="kebab-case"
            />
        </div>
        <div class="mb-3">
            <YakManInput label="Prioity" bind:value={prioity} placeholder="1" />
        </div>
        <div class="mb-3">
            <YakManInput
                label="Description"
                bind:value={description}
                placeholder="My cool label description "
            />
        </div>
        <div class="mb-3">
            <YakManInput
                label="Options"
                bind:value={options}
                placeholder="dev,prod"
            />
        </div>
        <YakManButton on:click={onCreateLabel}>Create</YakManButton>
    </YakManCard>
</div>
