<script lang="ts">
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import YakManInput from "$lib/components/YakManInput.svelte";

    let name = "";

    function onCreateProject() {
        console.log("on create", name);

        // TODO: FIX

        // match api::create_project(&name).await {
        //         Ok(()) => {
        //             let navigate = use_navigate(cx);
        //             let _ = navigate("/", Default::default());
        //         }
        //         Err(err) => error!("Error creating project: {}", err.to_string()),
        //     };
    }
</script>

<div class="container mx-auto">
    <YakManCard>
        <h1 class="text-lg font-bold mb-4">{"Add Project"}</h1>
        <div class="mb-3">
            <!-- TODO: Handle text masking -->
            <YakManInput
                label="Name"
                placeholder="my-project"
                bind:value={name}
            />
        </div>
        <YakManButton on:click={onCreateProject}>Create</YakManButton>
    </YakManCard>
</div>
