<script>
	import { page } from "$app/stores";
	import YakManHeader from "$lib/components/YakManHeader.svelte";
	import { trpc } from "$lib/trpc/client";
	import { onMount } from "svelte";
	import "./styles.css";
	import { roles } from "$lib/stores/roles";

	onMount(async () => {
		try {
			const userRoles = await trpc($page).oauth.fetchUserRoles.query();

			roles.set({
				globalRoles: userRoles.global_roles,
				roles: userRoles.roles,
			});
		} catch (e) {
			// TODO: Refresh Token
		}
	});
</script>

<div class="app">
	<YakManHeader />

	<main>
		<slot />
	</main>
</div>
