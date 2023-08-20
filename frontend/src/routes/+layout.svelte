<script>
	import YakManHeader from "$lib/components/YakManHeader.svelte";
	import { onMount, setContext } from "svelte";
	import { roles } from "$lib/stores/roles";
	import "./styles.css";
	import YakManModal from "$lib/components/YakManModal.svelte";
    import { globalModalState } from "$lib/stores/global-modal-state";

	// TODO: Fix typescript typing
	export let data;

	const userRoles = data.userRoles;

	$: {
		if (userRoles) {
			roles.set({
				globalRoles: userRoles.global_roles,
				roles: userRoles.roles,
			});
		}
	}

	onMount(() => {
		if (data.tokenRefreshNeeded) {
			fetch("/refresh-token", {
				method: "POST",
			}).then((response) => {
				if (response.status === 200) {
					window.location.reload();
				}
			});
		}
	});
</script>

<div class="app">
	<YakManHeader />

	<!-- Globally shared modal -->
	<YakManModal
		title={$globalModalState.title}
		open={$globalModalState.open}
		isStaticBackdrop={$globalModalState.isStaticBackdrop}
		onConfirm={$globalModalState.onConfirm}
	>
		<p class="text-gray-800">{$globalModalState.message}</p>
	</YakManModal>

	<main>
		<slot />
	</main>
</div>
