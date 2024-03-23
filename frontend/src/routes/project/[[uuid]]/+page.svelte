<script lang="ts">
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import { page } from "$app/stores";
    import { trpc } from "$lib/trpc/client";
    import { goto } from "$app/navigation";
    import type { PageData } from "./$types";
    import YakManSelect from "$lib/components/YakManSelect.svelte";
    import type { ModifyProjectPayload } from "$lib/trpc/routes/projects";
    import YakManCheckbox from "$lib/components/YakManCheckbox.svelte";
    import { openGlobaModal } from "$lib/stores/global-modal-state";

    export let data: PageData;

    type WebhookType = "slack";

    let projectUuid = $page.params.uuid;
    const isNewProject = !projectUuid;
    let name = data.project?.name ?? "";
    let webhookUrl = "";
    let webhookType: WebhookType = "slack";

    let isWebhookEnabled = false;
    let isInstanceCreateEventEnabled = false;
    let isInstanceUpdateEventEnabled = false;
    let isRevisionSubmittedEventEnabled = false;
    let isRevisionApprovedEventEnabled = false;
    let isRevisionRejectedEventEnabled = false;

    if (!isNewProject) {
        if (data.project?.notification_settings) {
            isWebhookEnabled = true;
            const notificationSettings = data.project?.notification_settings;

            if (notificationSettings.settings.Slack) {
                webhookType = "slack";
                webhookUrl = notificationSettings.settings.Slack.webhook_url;
            }

            const events = notificationSettings.events;
            isInstanceCreateEventEnabled = events.is_instance_created_enabled;
            isInstanceUpdateEventEnabled = events.is_instance_updated_enabled;
            isRevisionSubmittedEventEnabled =
                events.is_revision_submitted_enabled;
            isRevisionApprovedEventEnabled =
                events.is_revision_approved_enabled;
            isRevisionRejectedEventEnabled = events.is_revision_reject_enabled;
        }
    }

    const webhookUrlPlaceholder = {
        slack: "https://hooks.slack.com/services/...",
    } as const;

    $: isInvalid = (() => {
        if (!name || name.length === 0) {
            return true;
        }
        if (isWebhookEnabled) {
            if (!webhookUrl || webhookUrl.length === 0) {
                return true;
            }

            if (
                ![
                    isInstanceCreateEventEnabled,
                    isInstanceUpdateEventEnabled,
                    isRevisionSubmittedEventEnabled,
                    isRevisionApprovedEventEnabled,
                    isRevisionRejectedEventEnabled,
                ].includes(true)
            ) {
                return true;
            }
        }
        return false;
    })();

    async function onSave() {
        try {
            let createProjectPayload: ModifyProjectPayload = { name };

            if (webhookUrl?.length > 0) {
                switch (webhookType) {
                    case "slack": {
                        createProjectPayload.slack = {
                            webhookUrl: webhookUrl,
                        };
                    }
                }

                createProjectPayload.notificationEvents = {
                    isInstanceCreateEventEnabled: isInstanceCreateEventEnabled,
                    isInstanceUpdateEventEnabled: isInstanceUpdateEventEnabled,
                    isRevisionSubmittedEventEnabled:
                        isRevisionSubmittedEventEnabled,
                    isRevisionApprovedEventEnabled:
                        isRevisionApprovedEventEnabled,
                    isRevisionRejectedEventEnabled:
                        isRevisionRejectedEventEnabled,
                };
            }

            if (isNewProject) {
                const { projectUuid } =
                    await trpc($page).projects.createProject.mutate(
                        createProjectPayload,
                    );
                goto(`/?project=${projectUuid}`);
            } else {
                await trpc($page).projects.updateProject.mutate({
                    uuid: projectUuid!,
                    payload: createProjectPayload,
                });
                goto(`/?project=${projectUuid}`);
            }
        } catch (e) {
            console.error("Error creating project", e);
        }
    }

    function onDeleteClicked() {
        openGlobaModal({
            title: "Are you sure you want to delete this project?",
            message:
                "This cannot be undone. All configs for this project will also be deleted!",
            confirmButtonVariant: "danger",
            confirmButtonText: "Delete",
            async onConfirm() {
                try {
                    await trpc($page).projects.deleteProject.mutate(
                        projectUuid,
                    );
                    goto(`/`);
                } catch (e) {
                    console.error("Failed to delete project", e);
                }
            },
        });
    }
</script>

<div class="container mx-auto">
    <YakManCard>
        <h1 class="text-lg font-bold mb-4">
            {#if isNewProject}
                Add Project
            {:else}
                Modify Project
            {/if}
        </h1>
        <div class="mb-3">
            <YakManInput
                label="Name"
                placeholder="my-project"
                bind:value={name}
                mask="kebab-case"
            />
        </div>
    </YakManCard>

    <YakManCard extraClasses="mt-2">
        <h1 class="text-lg font-bold">Notification Settings (Webhooks)</h1>
        <div class="mb-4">
            <YakManCheckbox
                bind:value={isWebhookEnabled}
                label="Enable Notifications"
            />
        </div>
        {#if isWebhookEnabled}
            <div class="mb-3 flex gap-2">
                <YakManSelect
                    cotainerClasses="w-24"
                    label="Type"
                    bind:value={webhookType}
                >
                    <option value="slack">Slack</option>
                </YakManSelect>
                <YakManInput
                    label="URL"
                    placeholder={webhookUrlPlaceholder[webhookType]}
                    bind:value={webhookUrl}
                />
            </div>
            <div>
                <h3 class="text-md font-bold">Events</h3>
                <div class="flex flex-col">
                    <YakManCheckbox
                        bind:value={isInstanceCreateEventEnabled}
                        label="Instance Created"
                    />
                    <YakManCheckbox
                        bind:value={isInstanceUpdateEventEnabled}
                        label="Instance Updated"
                    />
                    <YakManCheckbox
                        bind:value={isRevisionSubmittedEventEnabled}
                        label="Revision Review Submitted"
                    />
                    <YakManCheckbox
                        bind:value={isRevisionApprovedEventEnabled}
                        label="Revision Review Approved"
                    />
                    <YakManCheckbox
                        bind:value={isRevisionRejectedEventEnabled}
                        label="Revision Review Rejected"
                    />
                </div>
            </div>
        {/if}
    </YakManCard>

    <YakManCard extraClasses="mt-2">
        <YakManButton on:click={onSave} type="submit" disabled={isInvalid}>
            {#if isNewProject}
                Create
            {:else}
                Update
            {/if}
        </YakManButton>
        {#if !isNewProject}
            <!-- TODO: Hide if not admin -->
            <YakManButton on:click={onDeleteClicked} variant="danger">
                Delete Project
            </YakManButton>
        {/if}
    </YakManCard>
</div>
