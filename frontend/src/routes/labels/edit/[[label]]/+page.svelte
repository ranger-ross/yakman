<script lang="ts">
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import { page } from "$app/stores";
    import { trpc } from "$lib/trpc/client";
    import { goto } from "$app/navigation";
    import type { PageData } from "./$types";
    import type { CreateLabel, UpdateLabel } from "$lib/trpc/routes/labels";

    export let data: PageData;
    const isEditMode = !!data.label;

    const label = isEditMode
        ? data.labels.find((l) => l.id === data.label)
        : null;

    // TODO: Maybe add an error/warning if the labelId is not found

    let name = label?.name ?? "";
    let description = label?.description ?? "";
    let options = label?.options.join(",") ?? "";

    async function onSave() {
        try {
            if (isEditMode) {
                const label: UpdateLabel = {
                    name: name,
                    description: description,
                    options: options
                        .split(",")
                        .filter((o) => !!o || o.length == 0),
                };

                await trpc($page).labels.updateLabel.mutate({
                    id: data.label!,
                    payload: label,
                });
                goto("/");
            } else {
                const label: CreateLabel = {
                    name: name,
                    description: description,
                    options: options
                        .split(",")
                        .filter((o) => !!o || o.length == 0),
                };

                await trpc($page).labels.createLabel.mutate(label);
                goto("/");
            }
        } catch (e) {
            console.error("Error creating config:", e);
        }
    }
</script>

<div class="container mx-auto">
    <YakManCard>
        <h1 class="text-lg font-bold mb-4">
            {#if isEditMode}
                Edit Label
            {:else}
                Add Label
            {/if}
        </h1>
        <div class="mb-3">
            <YakManInput
                label="Name"
                bind:value={name}
                placeholder="my-label-name"
                mask="kebab-case"
                disabled={isEditMode}
            />
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
        <YakManButton
            on:click={onSave}
            disabled={name.length === 0 || options.length == 0}
        >
            {#if isEditMode}
                Update
            {:else}
                Create
            {/if}
        </YakManButton>
    </YakManCard>
</div>
