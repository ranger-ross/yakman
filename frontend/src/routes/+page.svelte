<script lang="ts">
	import type {
		YakManConfig,
		YakManConfigInstance,
		YakManProject,
	} from "$lib/types/types";
	import LabelPill from "$lib/components/LabelPill.svelte";
	import StatusPill from "$lib/components/StatusPill.svelte";
	import YakManLink from "$lib/components/YakManLink.svelte";
	import YakManPopoverMenu from "$lib/components/YakManPopoverMenu.svelte";
	import YakManSelect from "$lib/components/YakManSelect.svelte";
	import type { PageData } from "./$types";
	import { goto } from "$app/navigation";
	import { page } from "$app/stores";
	import { trpc } from "$lib/trpc/client";
	import YakManModal from "$lib/components/YakManModal.svelte";
	import KebabMenuIcon from "$lib/icons/KebabMenuIcon.svelte";
	import YakManButton from "$lib/components/YakManButton.svelte";

	export let data: PageData;

	const projects: YakManProject[] = data.projects ?? [];

	let projectUuidFromQuery = $page.url.searchParams.get("project");
	let selectedProjectUuid = projectUuidFromQuery ?? projects[0]?.uuid;

	let selectedProject = projects.find((p) => p.uuid === selectedProjectUuid);

	let configToDelete: YakManConfig | null = null;

	function onProjectChange(e: Event) {
		const target = e?.currentTarget as HTMLSelectElement;
		const projectUuid = target?.value;
		selectedProject = projects.find((p) => p.uuid === projectUuid);
		console.log(selectedProject);
		goto(`?project=${selectedProject?.uuid}`);
	}

	function timeAgo(timestamp: number, locale = "en") {
		let value;
		const diff = (Date.now() - new Date(timestamp).getTime()) / 1000;
		const minutes = Math.floor(diff / 60);
		const hours = Math.floor(minutes / 60);
		const days = Math.floor(hours / 24);
		const months = Math.floor(days / 30);
		const years = Math.floor(months / 12);
		const rtf = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });

		if (years > 0) {
			value = rtf.format(0 - years, "year");
		} else if (months > 0) {
			value = rtf.format(0 - months, "month");
		} else if (days > 0) {
			value = rtf.format(0 - days, "day");
		} else if (hours > 0) {
			value = rtf.format(0 - hours, "hour");
		} else if (minutes > 0) {
			value = rtf.format(0 - minutes, "minute");
		} else {
			const roundedSeconds = Math.round(0 - diff);
			value = rtf.format(roundedSeconds, "second");
		}
		return value;
	}

	function getLastUpdatedTimestamp(instance: YakManConfigInstance): number {
		let last = instance.changelog[instance.changelog.length - 1];
		return last?.timestamp_ms;
	}

	async function onDeleteConfig() {
		// Optimistic update
		const index = data.configs.findIndex(
			(c) =>
				c.config.name === configToDelete?.name &&
				c.config.project_uuid === configToDelete.project_uuid,
		);
		const config = data.configs[index];
		data.configs.splice(index, 1);
		data = data; // Tell Svelte to re-render

		try {
			await trpc($page).configs.deleteConfig.mutate({
				name: configToDelete?.name!,
				projectUuid: configToDelete?.project_uuid!,
			});

			configToDelete = null;
		} catch (e) {
			// Rollback optimistic update
			data.configs.splice(index, 0, config);
			data = data; // Tell Svelte to re-render
		}
	}
</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="YakMan Configs" />
</svelte:head>

<section>
	<div class="container mx-auto">
		<YakManModal
			title="Delete Config"
			open={!!configToDelete}
			isStaticBackdrop={true}
			onConfirm={onDeleteConfig}
		>
			<p class="text-gray-800">Config Name: {configToDelete?.name}</p>
			<p class="text-gray-800">
				Are you sure want to delete this config?
			</p>
		</YakManModal>

		{#if projects.length === 0}
			<div class="bg-white border-2 border-gray-200 m-2 p-4">
				<div class="flex justify-center">
					<span class="text-gray-700">No projects created yet</span>
				</div>
			</div>
		{:else}
			<div class="flex items-end gap-2">
				<YakManSelect
					bind:value={selectedProjectUuid}
					label="Project"
					on:change={onProjectChange}
				>
					{#each projects as project}
						<option value={project.uuid}>{project.name}</option>
					{/each}
				</YakManSelect>

				<a
					tabindex="-1"
					href={`/add-config?project=${selectedProjectUuid}`}
				>
					<YakManButton>Add Config</YakManButton>
				</a>
			</div>

			{#if data.configs.length == 0}
				<div class="bg-white border-2 border-gray-200 m-2 p-4">
					<div class="flex justify-center">
						<span class="text-gray-700"
							>This project does not have any configs</span
						>
					</div>
				</div>
			{/if}

			{#each data.configs as config}
				<div class="bg-white border-2 border-gray-200 m-2 p-4 rounded">
					<div class="flex justify-between">
						<h3 class="text-gray-900 font-bold text-lg">
							{config.config.name}
						</h3>
						<YakManPopoverMenu
							options={[
								{ text: "Add Instance", value: "AddInstance" },
								{
									text: "Delete Config",
									value: "DeleteConfig",
								},
							]}
							on:select={(value) => {
								const selection = value.detail;
								if (selection === "AddInstance") {
									goto(
										`/modify-instance/${config.config.name}`,
									);
								} else if (selection === "DeleteConfig") {
									configToDelete = config.config;
								}
							}}
						>
							<KebabMenuIcon />
						</YakManPopoverMenu>
					</div>
					{#if config.metadata.length > 0}
						{#each config.metadata as instance}
							<div class="shadow-sm w-full h-1 mb-3" />
							<div class="flex justify-between">
								<div class="flex items-center gap-2">
									<div>
										<span class="font-bold"
											>{instance.instance}</span
										>
										<div class="text-gray-500">
											Last Updated: {timeAgo(
												getLastUpdatedTimestamp(
													instance,
												),
											)}
										</div>
									</div>
									<div class="flex flex-wrap gap-2">
										{#each instance.labels as label}
											<LabelPill
												text={`${label.label_type}=${label.value}`}
											/>
										{/each}
									</div>
								</div>
								<div class="flex items-center gap-5">
									{#if !!instance.pending_revision}
										<div>
											<StatusPill
												>Pending Changes</StatusPill
											>
										</div>
									{/if}
									<div class="flex flex-col items-end">
										<YakManLink
											href={`/modify-instance/${instance.config_name}/${instance.instance}`}
										>
											Edit
										</YakManLink>
										<YakManLink
											href={`/view-instance/${instance.config_name}/${instance.instance}`}
										>
											View
										</YakManLink>
										{#if !!instance.pending_revision}
											<YakManLink
												href={`/apply-changes/${instance.config_name}/${instance.instance}`}
											>
												Review Changes
											</YakManLink>
										{/if}
									</div>
								</div>
							</div>
						{/each}
					{:else}
						<div class="shadow-sm w-full h-1 mb-3" />
						<div class="flex justify-center">
							<span class="text-gray-700"
								>No config instances</span
							>
						</div>
					{/if}
				</div>
			{/each}
		{/if}
	</div>
</section>
