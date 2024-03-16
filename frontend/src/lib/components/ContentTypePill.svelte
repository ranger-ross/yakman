<script lang="ts">
    export let contentType = "";

    let styleClasses = "bg-gray-200 text-black";

    $: {
        if (contentType.includes("json")) {
            styleClasses = "bg-red-100 text-red-900";
        } else if (contentType.includes("yaml")) {
            styleClasses = "bg-blue-100 text-blue-900";
        } else {
            styleClasses = "bg-gray-200 text-black";
        }
    }
</script>

<div class="{styleClasses} text-md rounded-full pl-2 pr-2 pt-1 pb-1 w-fit">
    <p class="px-1">{contentType}</p>
</div>
