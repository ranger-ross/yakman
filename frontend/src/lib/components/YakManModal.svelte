<script lang="ts">
    import YakManButton from "./YakManButton.svelte";

    export let open: boolean = false;
    export let isStaticBackdrop: boolean = false;
    export let title = "";
    export let onConfirm = () => {};
    export let confirmButtonVariant: "primary" | "secondary" | "danger" = "primary";
    export let confirmButtonText: string = 'Confirm';

    let containerClass: string;
    let modalClass: string;

    $: {
        containerClass = open ? "opacity-100" : "opacity-0 pointer-events-none";
        modalClass = open ? "scale-100" : "scale-95";
    }
</script>

<div
    class="absolute top-0 left-0 h-full w-full z-40 transition-opacity {containerClass}"
>
    <div
        class="fixed z-10 inset-0 overflow-y-auto"
        aria-labelledby={title}
        role="dialog"
        aria-modal="true"
    >
        <div
            class="flex items-end justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0"
        >
            <!-- Backdrop -->
            <div
                class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity"
                aria-hidden="true"
                on:click={() => {
                    if (!isStaticBackdrop) {
                        open = false;
                    }
                }}
            />
            <!-- Spacer -->
            <span
                class="hidden sm:inline-block sm:align-middle sm:h-screen"
                aria-hidden="true"
            />
            <!-- Modal Window -->
            <div
                class="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full {modalClass}"
            >
                <div class="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
                    <div class="sm:flex sm:items-start">
                        <div
                            class="mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left"
                        >
                            <h3
                                class="text-lg font-bold leading-6 text-gray-900"
                            >
                                {title}
                            </h3>
                            <div class="mt-2">
                                <slot />
                            </div>
                        </div>
                    </div>
                </div>
                <div
                    class="bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse"
                >
                    <YakManButton
                        variant={confirmButtonVariant}
                        on:click={onConfirm}>{confirmButtonText}</YakManButton
                    >
                    <YakManButton
                        variant="secondary"
                        on:click={() => {
                            open = false;
                        }}
                    >
                        Cancel
                    </YakManButton>
                </div>
            </div>
        </div>
    </div>
</div>
