<script lang="ts">
    import { createEventDispatcher } from "svelte";
    const dispatch = createEventDispatcher();

    export let name: string = "";
    export let label: string = "";
    export let required: boolean = false;
    export let placeholder: string = "";
    export let value: string = "";
    export let disabled: boolean = false;
    export let mask: "" | "kebab-case" = "";
    export let type: "text" | "password" = "text";
    export let containerClass: string = "w-64";

    $: {
        switch (mask) {
            case "kebab-case":
                value = value.replace(/[^a-z0-9-]/g, "").toLowerCase();
                break;
        }
    }

    function onInput(e: any) {
        if (mask === "kebab-case") {
            let newValue = e.data;
            if (newValue) {
                let changed = false;
                if (newValue !== newValue.toLowerCase()) {
                    newValue = newValue.toLowerCase();
                    changed = true;
                }

                if (newValue.includes(" ")) {
                    console.log("space");
                    newValue = newValue.replaceAll(" ", "-");
                    changed = true;
                }

                if (changed) {
                    value = `${value}${newValue}`;
                }
            }
        }

        dispatch("input", e);
    }
</script>

<div class={containerClass}>
    <label class="block text-gray-700 text-sm font-bold mb-2" for={name}>
        {label}
    </label>
    <div class="relative">
        {#if type == "text"}
            <input
                type="text"
                class="block appearance-none w-full bg-white border border-gray-400 hover:border-indigo-500 px-4 py-2 pr-8 rounded shadow leading-tight focus:outline-none focus:shadow-outline transition-all duration-200"
                {name}
                {required}
                {placeholder}
                {disabled}
                bind:value
                on:input={onInput}
                on:focus={(e) => dispatch("focus", e)}
                on:blur={(e) => dispatch("blur", e)}
            />
        {:else if type == "password"}
            <input
                type="password"
                class="block appearance-none w-full bg-white border border-gray-400 hover:border-indigo-500 px-4 py-2 pr-8 rounded shadow leading-tight focus:outline-none focus:shadow-outline transition-all duration-200"
                {name}
                {required}
                {placeholder}
                {disabled}
                bind:value
                on:input={onInput}
                on:focus={(e) => dispatch("focus", e)}
                on:blur={(e) => dispatch("blur", e)}
            />
        {/if}
    </div>
</div>
