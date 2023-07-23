<script lang="ts">
    import { page } from "$app/stores";
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import { trpc } from "$lib/trpc/client";
    import { onMount } from "svelte";

    const LOCAL_STORAGE_OAUTH2_VERIFER_KEY = "oauth2-verifier";

    let redirectUri: string;

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

    onMount(async () => {
        let verifier = generateCodeVerifier();
        let challenge = await generateCodeChallengeFromVerifier(verifier);

        localStorage.setItem(LOCAL_STORAGE_OAUTH2_VERIFER_KEY, verifier);

        redirectUri = await trpc($page).generateOauthRedirectUri.mutate({
            challenge: challenge,
            challengeMethod: "S256",
        });
    });
</script>

<div class="container mx-auto">
    <YakManCard>
        <div class="flex flex-col items-center gap-4">
            <h1 class="text-xl font-bold">Login</h1>
            <a href={redirectUri}>
                <YakManButton>Click to login</YakManButton>
            </a>
        </div>
    </YakManCard>
</div>
