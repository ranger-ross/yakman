<script>
	import YakManHeader from "$lib/components/YakManHeader.svelte";
	import { onMount } from "svelte";
	import { roles } from "$lib/stores/roles";
	import "./styles.css";

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

	<main>
		<slot />
	</main>
</div>
