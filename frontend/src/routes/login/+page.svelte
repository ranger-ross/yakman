<script lang="ts">
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import { onMount } from "svelte";
    import type { PageData } from "./$types";
    import YakManInput from "$lib/components/YakManInput.svelte";

    export let data: PageData;

    const enableOauth = data.config.enable_oauth;

    const LOCAL_STORAGE_OAUTH2_VERIFER_KEY = "oauth2-verifier";

    let redirectUri: string;

    // Non OAuth login
    let error: string | null = data.error;
    let email = "";
    let password = "";

    // GENERATING CODE VERIFIER
    function dec2hex(dec: number) {
        return ("0" + dec.toString(16)).substr(-2);
    }
    function generateCodeVerifier() {
        var array = new Uint32Array(56 / 2);
        window.crypto.getRandomValues(array);
        return Array.from(array, dec2hex).join("");
    }

    // GENERATING CODE CHALLENGE FROM VERIFIER
    function sha256(plain: string) {
        const encoder = new TextEncoder();
        const data = encoder.encode(plain);
        return window.crypto.subtle.digest("SHA-256", data);
    }

    function base64urlencode(a: ArrayBuffer) {
        var str = "";
        var bytes = new Uint8Array(a);
        var len = bytes.byteLength;
        for (var i = 0; i < len; i++) {
            str += String.fromCharCode(bytes[i]);
        }
        return btoa(str)
            .replace(/\+/g, "-")
            .replace(/\//g, "_")
            .replace(/=+$/, "");
    }

    async function generateCodeChallengeFromVerifier(verifier: string) {
        var hashed = await sha256(verifier);
        var base64encoded = base64urlencode(hashed);
        return base64encoded;
    }

    async function onNonOAuthLogin() {
        fetch("/login", {
            method: "POST",
        });
    }

    onMount(async () => {
        if (enableOauth) {
            let verifier = generateCodeVerifier();
            let challenge = await generateCodeChallengeFromVerifier(verifier);

            localStorage.setItem(LOCAL_STORAGE_OAUTH2_VERIFER_KEY, verifier);

            const response = await fetch(`/login/init`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    challenge: {
                        challenge: challenge,
                        codeChallengeMethod: "S256",
                    },
                }),
            });

            const data = await response.json();

            redirectUri = data.redirectUrl;
        }
    });
</script>

<div class="container mx-auto">
    <YakManCard>
        <div class="flex flex-col items-center gap-4">
            <h1 class="text-xl font-bold">Login</h1>
            {#if enableOauth}
                <a href={redirectUri}>
                    <YakManButton>Login with SSO</YakManButton>
                </a>
            {:else}
                <form method="POST">
                    <div>
                        <YakManInput
                            required
                            name="username"
                            label="Email"
                            bind:value={email}
                        />
                    </div>
                    <div class="mt-2">
                        <YakManInput
                            required
                            name="password"
                            label="Password"
                            type="password"
                            bind:value={password}
                        />
                    </div>
                    <div class="mt-2">
                        {#if error}
                            <p class="text-red-600 font-semibold">
                                {error}
                            </p>
                        {/if}
                        <YakManButton
                            type="submit"
                            disabled={email.length == 0 || password.length == 0}
                        >
                            Login
                        </YakManButton>
                    </div>
                </form>
            {/if}
        </div>
    </YakManCard>
</div>
