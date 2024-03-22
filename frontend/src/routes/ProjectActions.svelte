<script lang="ts">
    import GearIcon from "$lib/icons/GearIcon.svelte";
    import PlusIcon from "$lib/icons/PlusIcon.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";

    export let projectUuid: string;
</script>

<YakManCard>
    <div class="flex items-center gap-2">
        <div class="bg-gray-100 bg-opacity-60 hover:bg-gray-200 rounded-lg p-2">
            <a tabindex="-1" href={`/add-config?project=${projectUuid}`}>
                <div class="w-4 h-4 fill-gray-800 hover:fill-black">
                    <PlusIcon />
                </div>
            </a>
        </div>
        <div class="bg-gray-100 bg-opacity-60 hover:bg-gray-200 rounded-lg p-2">
            <a tabindex="-1" href={`/project/${projectUuid}`}>
                <div class="w-4 h-4 fill-gray-800 hover:fill-black">
                    <GearIcon />
                </div>
            </a>
        </div>
    </div>
</YakManCard>
