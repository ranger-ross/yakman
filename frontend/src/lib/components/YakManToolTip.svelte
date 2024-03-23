<script lang="ts">
    export let message: string;
    export let spacing = '-translate-y-[-2rem]';
</script>

<div class="group relative">
    <div class="hidden group-hover:flex justify-center z-50">
        <div class="absolute {spacing}">
            <div class="bg-gray-500 w-max rounded py-0.5 px-2">
                <p class="text-white">{message}</p>
            </div>
        </div>
    </div>
    <slot />
</div>


