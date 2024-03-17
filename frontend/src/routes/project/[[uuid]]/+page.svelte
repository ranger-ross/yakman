<script lang="ts">
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import { page } from "$app/stores";
    import { trpc } from "$lib/trpc/client";
    import { goto } from "$app/navigation";
    import type { PageData } from "./$types";
    import YakManSelect from "$lib/components/YakManSelect.svelte";
    import type { CreateProjectPayload } from "$lib/trpc/routes/projects";

    export let data: PageData;

    type WebhookType = "slack";

    let projectId = $page.params.uuid;
    const isNewProject = !projectId;
    let name = data.project?.name ?? "";
    let webhookUrl = "";
    let webhookType: WebhookType = "slack";

    const webhookUrlPlaceholder = {
        slack: "https://hooks.slack.com/services/...",
    } as const;

    async function onSave() {
        if (isNewProject) {
            onCreateProject();
        } else {
            // TODO: Implment
            console.error("UPDATE PROJECT NOT IMPLMENTED");
        }
    }

    async function onCreateProject() {
        try {
            let createProjectPayload: CreateProjectPayload = { name };

            if (webhookUrl?.length > 0) {
                switch (webhookType) {
                    case "slack": {
                        createProjectPayload.slack = {
                            webhookUrl: webhookUrl,
                        };
                    }
                }
            }

            const { projectUuid } =
                await trpc($page).projects.createProject.mutate(
                    createProjectPayload,
                );
            goto(`/?project=${projectUuid}`);
        } catch (e) {
            console.error("Error creating project", e);
        }
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
                disabled={!isNewProject}
                mask="kebab-case"
            />
        </div>
    </YakManCard>

    <YakManCard extraClasses="mt-2">
        <h1 class="text-lg font-bold mb-4">Notification Settings (Webhooks)</h1>
        <div class="mb-3 flex gap-2">
            <YakManInput
                label="URL"
                placeholder={webhookUrlPlaceholder[webhookType]}
                bind:value={webhookUrl}
            />
            <YakManSelect
                cotainerClasses="w-24"
                label="Type"
                bind:value={webhookType}
            >
                <option value="slack">Slack</option>
            </YakManSelect>
        </div>
    </YakManCard>

    <YakManCard extraClasses="mt-2">
        <YakManButton
            on:click={onSave}
            type="submit"
            disabled={!name || name.length === 0}
        >
            {#if isNewProject}
                Create
            {:else}
                Update
            {/if}
        </YakManButton>
    </YakManCard>
</div>
