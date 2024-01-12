<script lang="ts">
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";

    let error: string;

    const LOCAL_STORAGE_OAUTH2_VERIFER_KEY = "oauth2-verifier";

    onMount(async () => {
        const verifier = localStorage.getItem(LOCAL_STORAGE_OAUTH2_VERIFER_KEY);
        const urlParams = new URLSearchParams(window.location.search);
        const code = urlParams.get("code");
        const state = urlParams.get("state");

        try {
            await fetch("/session/oauth-callback", {
                method: "POST",
                body: JSON.stringify({
                    code: code!,
                    state: state!,
                    verifier: verifier!,
                }),
            });

            goto("/");
        } catch (e) {
            console.error(e);
            error = e as string;
        }
    });
</script>

<div style="display: flex; justify-content: center">
    <div>
        {#if error}
            {error}
            <br />
            <a href="/login">"Back to Login"</a>
        {:else}
            Logging in...
        {/if}
    </div>
</div>
