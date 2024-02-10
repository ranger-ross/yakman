<script lang="ts">
    import { replaceState } from "$app/navigation";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManSegmentSelect from "$lib/components/YakManSegmentSelect.svelte";
    import type { PageData } from "./$types";
    import ApiKeyTab from "./ApiKeyTab.svelte";
    import UsersTab from "./UsersTab.svelte";

    export let data: PageData;

    let selectedHistoryTab: "Users" | "Api Keys" = data.tab ?? "Users";

    function onTabChange(option: string) {
        replaceState(`?tab=${option}`, {});
    }
</script>

<div class="container mx-auto">
    <h1 class="text-xl font-bold">Admin</h1>

    <YakManCard>
        <YakManSegmentSelect
            bind:selectedOption={selectedHistoryTab}
            options={["Users", "Api Keys"]}
            on:select={(event) => onTabChange(event.detail)}
        />
    </YakManCard>

    {#if selectedHistoryTab == "Users"}
        <UsersTab />
    {/if}

    {#if selectedHistoryTab == "Api Keys"}
        <ApiKeyTab />
    {/if}
</div>
