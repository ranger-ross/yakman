<script lang="ts">
    export let contentType = "";

    function getStyleClasses() {
        if (contentType.includes("json")) {
            return "bg-red-100 text-red-900";
        }

        if (contentType.includes("yaml")) {
            return "bg-blue-100 text-blue-900";
        }

        return "bg-gray-200 text-black";
    }
</script>

<div class="{getStyleClasses()} text-md rounded-full pl-2 pr-2 pt-1 pb-1 w-fit">
    <p class="px-1">{contentType}</p>
</div>
