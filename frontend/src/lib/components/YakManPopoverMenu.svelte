<script lang="ts">
    import KebabMenuIcon from "$lib/icons/KebabMenuIcon.svelte";
    import { createEventDispatcher } from "svelte";

    const dispatch = createEventDispatcher();

    const OPENED_CLASSES = "transform opacity-100 scale-100";
    const CLOSED_CLASSES = "transform opacity-0 scale-95";

    type PopoverMenuOption = {
        text: string;
        value: string;
    };

    export let open: boolean = false;
    export let options: PopoverMenuOption[] = [];
    let extra_class = CLOSED_CLASSES;
    let popoverElementRef: HTMLElement;

    function onModalChange(isOpen: boolean) {
        if (isOpen) {
            open = isOpen;

            // Add a slight delay to wait for the elements to be added to the DOM
            // so that when the classes are added the animation plays properly
            setTimeout(() => {
                extra_class = OPENED_CLASSES;
            }, 1);
        } else {
            extra_class = CLOSED_CLASSES;
            setTimeout(
                () => {
                    open = isOpen;
                },
                100 // This duration should match the duration Tailwind class below
            );
        }
    }

    function onSelect(value: string) {
        onModalChange(false);
        dispatch("select", value);
    }

    function outsideClick(e: MouseEvent) {
        if (open && !popoverElementRef.contains(e.target as Node)) {
            onModalChange(false);
        }
    }
</script>

<svelte:window on:click={outsideClick} />

<div class="relative inline-block text-left" bind:this={popoverElementRef}>
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <!-- svelte-ignore a11y-missing-attribute -->
    <div class="cursor-pointer" on:click={(_) => onModalChange(!open)}>
        <KebabMenuIcon />
    </div>

    {#if open}
        <div
            class="origin-top-right absolute right-0 mt-2 w-56 rounded-md shadow-lg ring-1 ring-black bg-white ring-opacity-5 transition ease-out duration-100 {extra_class}"
        >
            <div
                class="py-1"
                role="menu"
                aria-orientation="vertical"
                aria-labelledby="options-menu"
            >
                {#each options as option}
                    <!-- svelte-ignore a11y-click-events-have-key-events -->
                    <!-- svelte-ignore a11y-no-static-element-interactions -->
                    <!-- svelte-ignore a11y-missing-attribute -->
                    <!-- svelte-ignore a11y-interactive-supports-focus -->
                    <a
                        class="cursor-pointer block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 hover:text-gray-900"
                        role="menuitem"
                        on:click={(e) => onSelect(option.value)}
                    >
                        {option.text}
                    </a>
                {/each}
            </div>
        </div>
    {/if}
</div>
