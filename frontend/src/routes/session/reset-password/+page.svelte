<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";
    import { trpc } from "$lib/trpc/client";
    import type { PageData } from "./$types";

    export let data: PageData;

    let password = "";
    let confirmPassword = "";
    let isResetDisabled = true;
    let passwordError: string | null = null;

    $: {
        function getPasswordError(): string | null {
            if (password != confirmPassword) {
                return "Passwords do not match";
            }

            // If the password is empty do not show an error yet
            if (password.length === 0) {
                return null;
            }

            if (password.length < 9) {
                return "Password must be at least 9 characters";
            }

            if (password.length > 100) {
                return "Password must be less than 100 charaters";
            }

            return null;
        }

        passwordError = getPasswordError();
        isResetDisabled = passwordError != null || password.length === 0;
    }

    async function resetPassword() {
        if (isResetDisabled) {
            return;
        }

        try {
            await trpc($page).auth.resetPassword.mutate({
                id: data.id,
                userUuid: data.userUuid,
                password,
            });
            goto("/");
        } catch (e) {
            console.error("failed to reset password", e);
        }
    }
</script>

<div class="container mx-auto">
    <YakManCard>
        <div class="flex justify-center">
            <div>
                <h1 class="text-lg font-bold mb-2">Reset Password</h1>

                {#if data.isValidLink}
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

                        {#if !!passwordError}
                            <div class="text-red-600 font-semibold">
                                {passwordError}
                            </div>
                        {:else}
                            <div class="mb-6"></div>
                        {/if}

                        <YakManButton
                            on:click={resetPassword}
                            disabled={isResetDisabled}
                        >
                            Reset Password
                        </YakManButton>
                    {/if}
                {:else}
                    <p>Invalid password reset link</p>
                {/if}
            </div>
        </div>
    </YakManCard>
</div>
