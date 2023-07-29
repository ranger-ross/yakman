<script>
	import { page } from "$app/stores";
	import YakManHeader from "$lib/components/YakManHeader.svelte";
	import { trpc } from "$lib/trpc/client";
	import { onMount } from "svelte";
	import "./styles.css";
	import { roles } from "$lib/stores/roles";
	import { goto } from "$app/navigation";

	onMount(async () => {
		try {
			const userRoles = await trpc($page).oauth.fetchUserRoles.query();

			if (
				userRoles.global_roles.length == 0 &&
				Object.keys(userRoles.roles).length == 0
			) {
				throw Error("No Roles found");
			}

			roles.set({
				globalRoles: userRoles.global_roles,
				roles: userRoles.roles,
			});
		} catch (e) {
			const response = await fetch("/refresh-token", {
				method: "POST",
			});
			if (response.status === 200) {
				goto(window.location.pathname);
			}
		}
	});
</script>

<div class="app">
	<YakManHeader />

	<main>
		<slot />
	</main>
</div>
