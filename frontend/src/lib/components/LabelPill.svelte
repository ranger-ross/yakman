<script lang="ts">
    function colorFromString(input: string) {
        let colors = [
            ["bg-red-100", "text-red-900"],
            ["bg-yellow-100", "text-yellow-900"],
            ["bg-green-100", "text-green-900"],
            ["bg-blue-100", "text-blue-900"],
            ["bg-indigo-100", "text-indigo-900"],
            ["bg-purple-100", "text-purple-900"],
            ["bg-pink-100", "text-pink-900"],
        ];

        // TODO: Create a better way to color labels
        function hash(input: string): number {
            var hash = 0,
                i,
                chr;
            if (input.length === 0) return hash;
            for (i = 0; i < input.length; i++) {
                chr = input.charCodeAt(i);
                hash = (hash << 5) - hash + chr;
                hash |= 0; // Convert to 32bit integer
            }
            return hash < 0 ? hash * -1 : hash; // if negative, make positive
        }

        const index = hash(input) % colors.length;
        const [bgColorClass, textColorClass] = colors[index];

        return {
            bgColorClass: bgColorClass,
            textColorClass: textColorClass,
        };
    }

    export let text: string = "";

    let { bgColorClass, textColorClass } = colorFromString(text);

    // When text changes, update the colors
    $: {
        let newColors = colorFromString(text);
        bgColorClass = newColors.bgColorClass;
        textColorClass = newColors.textColorClass;
    }
</script>

<div
    class="{bgColorClass} {textColorClass} text-sm rounded-full pl-2 pr-2 pt-1 pb-1 w-fit"
>
    {text}
</div>
