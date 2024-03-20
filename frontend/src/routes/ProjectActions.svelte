<script lang="ts">
    import GearIcon from "$lib/icons/GearIcon.svelte";
    import PlusIcon from "$lib/icons/PlusIcon.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";

    export let projectUuid: string;
</script>

<YakManCard>
    <div class="flex items-center gap-2">
        <div class="fill-gray-700 hover:fill-black">
            <a tabindex="-1" href={`/add-config?project=${projectUuid}`}>
                <PlusIcon />
            </a>
        </div>
        <div class="fill-gray-700 hover:fill-black">
            <a tabindex="-1" href={`/project/${projectUuid}`}>
                <GearIcon />
            </a>
        </div>
    </div>
</YakManCard>
