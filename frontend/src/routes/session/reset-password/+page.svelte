<script lang="ts">
    import type { PageData } from "./$types";

    export let data: PageData;

    console.log(data);
</script>

<div>reset password</div>

<div>
    id: {data.id}
</div>

<div>
    User UUID: {data.userUuid}
</div>
