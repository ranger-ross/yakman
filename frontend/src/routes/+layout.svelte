<script lang="ts">
	import YakManHeader from "$lib/components/YakManHeader.svelte";
	import { onMount } from "svelte";
	import { roles } from "$lib/stores/roles";
	import "./styles.css";
	import YakManModal from "$lib/components/YakManModal.svelte";
	import { globalModalState } from "$lib/stores/global-modal-state";
	import { goto } from "$app/navigation";
	import { userInfo } from "$lib/stores/user-info";
	import YakManCard from "$lib/components/YakManCard.svelte";
	import { page } from "$app/stores";

	export let data;

	const pagesWithNoRefreshTokenNeeded: string[] = ["/session/oauth-callback"];

	$: {
		const userRoles = data.userRoles;
		if (userRoles) {
			roles.set({
				globalRoles: userRoles.global_roles,
				roles: userRoles.roles,
			});
			userInfo.set({
				profilePictureUrl: userRoles.profile_picture,
			});
		}
	}

	onMount(() => {
		if (
			data.tokenRefreshNeeded &&
			!pagesWithNoRefreshTokenNeeded.includes($page.url.pathname)
		) {
			fetch("/session/refresh-token", {
				method: "POST",
			}).then((response) => {
				if (response.status === 200) {
					window.location.reload();
				} else if (response.status === 401) {
					goto(`/login`);
				}
			});
		}
	});
</script>

<svelte:head>
	<title>YakMan</title>
	<meta name="description" content="YakMan Configs" />
</svelte:head>

<div class="app">
	<YakManHeader />

	<!-- Globally shared modal -->
	<YakManModal
		title={$globalModalState.title}
		open={$globalModalState.open}
		isStaticBackdrop={$globalModalState.isStaticBackdrop}
		onConfirm={$globalModalState.onConfirm}
		confirmButtonVariant={$globalModalState.confirmButtonVariant}
		confirmButtonText={$globalModalState.confirmButtonText}
	>
		<p class="text-gray-800">{$globalModalState.message}</p>
	</YakManModal>

	<main>
		{#if data.tokenRefreshNeeded && !pagesWithNoRefreshTokenNeeded.includes($page.url.pathname)}
			<div class="container mx-auto">
				<YakManCard>
					<p class="text-center">Refreshing session...</p>
				</YakManCard>
			</div>
		{:else}
			<slot />
		{/if}
	</main>
</div>
