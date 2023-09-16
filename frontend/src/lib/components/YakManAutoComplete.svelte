<script lang="ts">
    import YakManInput from "./YakManInput.svelte";

    export let label = "";
    export let placeholder = "";
    export let disabled = false;
    export let options: string[] = [];
    export let value = "";

    let filteredOptions: string[] = [];
    let isFocused = false;

    function updateOptions() {
        filteredOptions = options.filter((option) =>
            option.toLowerCase().includes(value.toLowerCase())
        );
    }

    function handleSelect(option: string) {
        value = option;
        updateOptions()
    }

    function handleFocus(_isFocused: boolean) {
        if (_isFocused) {
            isFocused = _isFocused;
        } else {
            // Wait for selection to complete before hiding.
            // This is not a great solution but it works for now.
            setTimeout(() => isFocused = _isFocused, 150);
        }
    }
</script>

<div class="w-64">
    <YakManInput
        {label}
        {placeholder}
        {disabled}
        bind:value
        on:input={updateOptions}
        on:focus={() => handleFocus(true)}
        on:blur={() => handleFocus(false)}
    />
    {#if isFocused && value && filteredOptions.length}
        <div
            class="absolute z-10 mt-2 w-fit bg-white border border-gray-400 rounded shadow-lg"
        >
            {#each filteredOptions as option (option)}
                <div
                    class="cursor-pointer px-4 py-2 hover:bg-indigo-500 hover:text-white"
                    on:click={() => handleSelect(option)}
                >
                    {option}
                </div>
            {/each}
        </div>
    {/if}
</div>
