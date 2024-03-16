<script lang="ts">
    export let label: string = "";
    export let value: boolean = false;
</script>

<label class="inline-flex items-center mt-3">
    <input
        type="checkbox"
        class="form-checkbox h-4 w-4 accent-indigo-600 transition duration-150 ease-in-out"
        bind:checked={value}
    />
    <span class="ml-2 text-md text-gray-700">{label}</span>
</label>
