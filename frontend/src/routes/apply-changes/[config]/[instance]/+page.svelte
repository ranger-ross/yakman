<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import ApprovedPill from "$lib/components/ApprovedPill.svelte";
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import { openGlobaModal } from "$lib/stores/global-modal-state";
    import { trpc } from "$lib/trpc/client";
    import type { PageData } from "./$types";
    import ConfigPreview from "./ConfigPreview.svelte";

    export let data: PageData;

    let { config, instance } = $page.params;

    function onApprove(isApply: boolean) {
        const message =
            "Are you sure you want to approve this change?" +
            (isApply
                ? "It will be applied immediately"
                : "You will need to apply it before changes are reflected");

        openGlobaModal({
            title: "Approve Changes",
            message,
            async onConfirm() {
                try {
                    await trpc($page).revisions.reviewInstanceRevision.mutate({
                        configName: config,
                        instance: instance,
                        revision: data.pendingRevision?.revision as string,
                        reviewResult: isApply ? "ApproveAndApply" : "Approve",
                    });

                    if (isApply) {
                        goto(`/view-instance/${config}/${instance}`);
                    } else {
                        goto(`/apply-changes/${config}/${instance}`, {
                            invalidateAll: true,
                        });
                    }
                } catch (e) {
                    console.error("Error while approving config: ", e);
                }
            },
        });
    }

    async function onReject() {
        openGlobaModal({
            title: "Reject Changes",
            message: "Are you sure you want to reject these changes?",
            async onConfirm() {
                try {
                    await trpc($page).revisions.reviewInstanceRevision.mutate({
                        configName: config,
                        instance: instance,
                        revision: data.pendingRevision?.revision as string,
                        reviewResult: "Reject",
                    });
                    goto(`/view-instance/${config}/${instance}`);
                } catch (e) {
                    console.error("Error while approving config: ", e);
                }
            },
        });
    }

    async function onApply() {
        openGlobaModal({
            title: "Approve Changes",
            message: "Are you sure you want to apply these changes?",
            async onConfirm() {
                try {
                    await trpc($page).revisions.applyInstanceRevision.mutate({
                        configName: config,
                        instance: instance,
                        revision: data.pendingRevision?.revision as string,
                    });
                    goto(`/view-instance/${config}/${instance}`);
                } catch (e) {
                    console.error("Error while approving config: ", e);
                }
            },
        });
    }
</script>

<div class="container mx-auto">
    <YakManCard>
        <h1 class="text-xl font-bold mb-3">
            Apply Config {config} -> {instance}
        </h1>
        {#if data.pendingRevision}
            <div>
                <div class="flex gap-2">
                    <h3 class="text-md font-bold text-gray-600">
                        Pending Revision => {data.pendingRevision?.revision}
                    </h3>

                    {#if data.pendingRevision.review_state == "Approved"}
                        <ApprovedPill />
                    {/if}
                </div>
            </div>
        {:else}
            No pending revisions
        {/if}
    </YakManCard>

    {#if data.pendingRevision}
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6 my-2">
            <YakManCard>
                <ConfigPreview
                    title="Current"
                    contentType={data.currentData?.contentType ?? ""}
                    data={data.currentData?.data ?? ""}
                />
            </YakManCard>
            <YakManCard>
                <ConfigPreview
                    title="New"
                    contentType={data.pendingData?.contentType ?? ""}
                    data={data.pendingData?.data ?? ""}
                />
            </YakManCard>
        </div>
    {/if}
    {#if data.pendingRevision}
        <YakManCard>
            <h1 class="text-lg font-bold mb-1">Actions</h1>
            <YakManButton variant="secondary" on:click={() => onReject()}>
                Reject
            </YakManButton>

            {#if data.pendingRevision.review_state != "Approved"}
                <YakManButton on:click={() => onApprove(false)}>
                    Approve
                </YakManButton>

                <YakManButton on:click={() => onApprove(true)}>
                    Approve and Apply
                </YakManButton>
            {:else if data.pendingRevision.review_state == "Approved"}
                <YakManButton on:click={onApply}>Apply</YakManButton>
            {/if}
        </YakManCard>
    {/if}
</div>
