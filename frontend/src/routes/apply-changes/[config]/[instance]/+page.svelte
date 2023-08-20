<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import { openGlobaModal } from "$lib/stores/global-modal-state";
    import { trpc } from "$lib/trpc/client";
    import type { PageData } from "./$types";

    export let data: PageData;

    let { config, instance } = $page.params;

    function onApprove() {
        openGlobaModal({
            title: "Approve Changes",
            message: "Are you sure you want to approve this change? It will be applied immediately",
            onConfirm() {
                saveChanges();
            },
        });
    }

    async function saveChanges() {
        try {
            await trpc($page).revisions.approveInstanceRevision.mutate({
                configName: config,
                instance: instance,
                revision: data.pendingRevision as string,
            });
            goto(`/view-instance/${config}/${instance}`);
        } catch (e) {
            console.error("Error while approving config: ", e);
        }
    }
</script>

<div class="container mx-auto">
    <YakManCard>
        <h1 class="text-xl font-bold mb-3">
            Apply Config {config} -> {instance}
        </h1>
        {#if data.pendingRevision}
            <div>
                <h3 class="text-md font-bold text-gray-600">
                    Pending Revision => {data.pendingRevision}
                </h3>

                <div class="w-full flex justify-evenly gap-6">
                    <div class="m-2 p-2 bg-gray-100 rounded-md w-80">
                        <div class="text-lg font-bold mb-3">Current</div>
                        <div class="text-md font-bold mb-1">Content Type</div>
                        <div class="text-md mb-2">
                            {data.currentData?.contentType}
                        </div>
                        <div class="text-md font-bold mb-1">Text</div>
                        <div>{data.currentData?.data}</div>
                    </div>
                    <div class="m-2 p-2 bg-gray-100 rounded-md w-80">
                        <div class="text-lg font-bold mb-3">New</div>
                        <div class="text-md font-bold mb-1">Content Type</div>
                        <div class="text-md mb-2">
                            {data.pendingData?.contentType}
                        </div>
                        <div class="text-md font-bold mb-1">Text</div>
                        <div>{data.pendingData?.data}</div>
                    </div>
                </div>

                <YakManButton on:click={onApprove}>Approve</YakManButton>
            </div>
        {:else}
            No pending revisions
        {/if}
    </YakManCard>
</div>
