<script lang="ts">
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import type { PageData } from "./$types";

    export let data: PageData;

    let password = "";
    let confirmPassword = "";

    // TODO: Verify the id and user id are valid and show and error if they are not

    function resetPassword() {
        if (password != confirmPassword) {
            return;
        }

        console.warn("TODO: send password reset request");
    }
</script>

<div class="container mx-auto">
    <YakManCard>
        <div class="flex justify-center">
            <div>
                <h1 class="text-lg font-bold mb-2">Reset Password</h1>

                {#if !data.id && !data.userUuid}
                    <p>Error</p>
                {:else}
                    <YakManInput
                        placeholder="New password"
                        bind:value={password}
                        type="password"
                    />
                    <YakManInput
                        placeholder="Enter password again"
                        bind:value={confirmPassword}
                        type="password"
                    />

                    <YakManButton on:click={resetPassword}>
                        Reset Password
                    </YakManButton>
                {/if}
            </div>
        </div>
    </YakManCard>
</div>
