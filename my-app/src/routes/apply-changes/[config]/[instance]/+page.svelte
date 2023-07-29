<script lang="ts">
    import { page } from "$app/stores";
    import YakManButton from "$lib/components/YakManButton.svelte";
    import YakManCard from "$lib/components/YakManCard.svelte";
    import type { PageData } from "./$types";

    export let data: PageData;

    let { config, instance } = $page.params;

    async function onApprove() {}
</script>

<div class="container mx-auto">
    <YakManCard>
        <h1 class="text-xl font-bold mb-3">
            Apply Config {config} -> {instance}
        </h1>
        {#if data.pendingRevision}
            <div>
                <h3 class="text-md font-bold text-gray-600">
                    Pending Revision => {data.pendingRevision}
                </h3>

                <div class="w-full flex justify-evenly gap-6">
                    <div class="m-2 p-2 bg-gray-100 rounded-md w-80" >
                        <div class="text-lg font-bold mb-3">Current</div>
                        <div class="text-md font-bold mb-1">Content Type</div>
                        <div class="text-md mb-2">{data.currentData?.contentType}</div>
                        <div class="text-md font-bold mb-1">Text</div>
                        <div>{data.currentData?.data}</div>
                    </div>
                    <div class="m-2 p-2 bg-gray-100 rounded-md w-80" >
                        <div class="text-lg font-bold mb-3">New</div>
                        <div class="text-md font-bold mb-1">Content Type</div>
                        <div class="text-md mb-2">{data.pendingData?.contentType}</div>
                        <div class="text-md font-bold mb-1">Text</div>
                        <div>{data.pendingData?.data}</div>
                    </div>
                </div>

                <YakManButton on:click={onApprove}>Approve</YakManButton>
            </div>
        {:else}
            No pending revisions
        {/if}
    </YakManCard>
</div>

<!--

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct ApplyConfigPageData {
    revisions: Vec<ConfigInstanceRevision>,
    pending_revision: Option<String>,
    pending_revision_data: Option<(String, String)>,
    current_revision_data: Option<(String, String)>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct DiffData {
    original: (String, String),
    new: (String, String),
}

#[component]
pub fn apply_config_page(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);

    // TODO: use a better way to extract params
    let config_name = move || params.with(|params| params.get("config_name").cloned().unwrap());
    let instance = move || params.with(|params| params.get("instance").cloned().unwrap());

    let page_data = create_resource(
        cx,
        move || (config_name(), instance()),
        |(config_name, instance)| async move {
            let mut revsions: Vec<ConfigInstanceRevision> = vec![];
            let mut pending_revision: Option<String> = None;
            let mut instance_metadata: Option<ConfigInstance> = None;
            let mut current_data: Option<(String, String)> = None;
            let mut pending_data: Option<(String, String)> = None;

            if let Ok(data) = api::fetch_instance_revisions(&config_name, &instance).await {
                revsions = data;
            }

            let metadata = api::fetch_config_metadata(&config_name).await; // TODO: add a instance query param to avoid over fetching data

            for inst in metadata {
                if inst.instance == instance {
                    pending_revision = inst.pending_revision.clone();
                    instance_metadata = Some(inst);
                }
            }

            if let Some(instance_metadata) = instance_metadata {
                let current_rev = instance_metadata.current_revision;
                let pending_rev = instance_metadata.pending_revision.unwrap();

                current_data = api::fetch_revision_data(&config_name, &instance, &current_rev)
                    .await
                    .ok();

                pending_data = api::fetch_revision_data(&config_name, &instance, &pending_rev)
                    .await
                    .ok();
            }

            ApplyConfigPageData {
                revisions: revsions,
                pending_revision: pending_revision,
                pending_revision_data: pending_data,
                current_revision_data: current_data,
            }
        },
    );

    let pending_revision = move || page_data.read(cx).unwrap().pending_revision.unwrap();

    let on_approve = move |_| {
        spawn_local(async move {
            match api::approve_instance_revision(&config_name(), &instance(), &pending_revision())
                .await
            {
                Ok(()) => {
                    let navigate = use_navigate(cx);
                    let _ = navigate(
                        &format!("/view-instance/{}/{}", config_name(), instance()),
                        Default::default(),
                    );
                }
                Err(e) => error!("Error while approving config: {}", e.to_string()),
            };
        })
    };

    let original_text = move || {
        page_data.read(cx).map(|d| {
            d.current_revision_data
                .map(|d| d.0)
                .unwrap_or("".to_string())
        })
    };

    let new_text = move || {
        page_data.read(cx).map(|d| {
            d.pending_revision_data
                .map(|d| d.0)
                .unwrap_or("".to_string())
        })
    };

    view! { cx,
        <div class="container mx-auto">
            <YakManCard>
                <h1 class="text-xl font-bold mb-3">
                    {"Apply Config "} {config_name} {" -> "} {instance}
                </h1>
                {move || match page_data.read(cx) {
                    Some(data) => {
                        view! { cx,
                            {move || match &data.pending_revision {
                                Some(pending_revision) => {
                                    view! { cx,
                                        <div>
                                            <h3 class="text-md font-bold text-gray-600">
                                                {"Pending Revision => "} {pending_revision}
                                            </h3>
                                            <ConfigDiffs
                                                original=original_text().unwrap_or("Loading".to_string())
                                                new=new_text().unwrap_or("Loading".to_string())
                                            />
                                            <YakManButton on:click=on_approve>{"Approve"}</YakManButton>
                                        </div>
                                    }
                                        .into_view(cx)
                                }
                                None => {
                                    view! { cx, "No pending revisions" }
                                        .into_view(cx)
                                }
                            }}
                        }
                            .into_view(cx)
                    }
                    None => {
                        view! { cx, <p>"Loading..."</p> }
                            .into_view(cx)
                    }
                }}
            </YakManCard>
        </div>
    }
}

#[derive(Debug, Clone)]
enum TextColor {
    Regular,
    Green,
    StrongGreen,
    Red,
}

impl TextColor {
    fn styles(&self) -> String {
        match self {
            TextColor::Regular => String::from(""),
            TextColor::Green => String::from("color: darkgreen"),
            TextColor::StrongGreen => String::from("color: lime"),
            TextColor::Red => String::from("color: red"),
        }
    }
}

#[component]
fn config_diffs(cx: Scope, #[prop()] original: String, #[prop()] new: String) -> impl IntoView {
    let grouped_by_lines = move || {
        let Changeset { diffs, .. } = Changeset::new(&original, &new, "\n");

        let mut grouped_by_lines: Vec<Vec<(String, TextColor)>> = vec![];

        for i in 0..diffs.len() {
            match diffs[i] {
                Difference::Same(ref x) => {
                    grouped_by_lines.push(vec![(x.clone(), TextColor::Regular)]);
                }
                Difference::Add(ref x) => {
                    let mut changes = vec![];

                    match diffs[i - 1] {
                        Difference::Rem(ref y) => {
                            let Changeset { diffs, .. } = Changeset::new(y, x, " ");
                            for c in diffs {
                                match c {
                                    Difference::Same(ref z) => {
                                        changes.push((z.clone(), TextColor::Green));
                                    }
                                    Difference::Add(ref z) => {
                                        changes.push((z.clone(), TextColor::StrongGreen));
                                    }
                                    _ => (),
                                }
                            }
                        }
                        _ => {
                            changes.push((x.clone(), TextColor::Green));
                        }
                    };
                    grouped_by_lines.push(changes);
                }
                Difference::Rem(ref x) => {
                    grouped_by_lines.push(vec![(x.clone(), TextColor::Red)]);
                }
            }
        }

        grouped_by_lines
    };

    view! { cx,
        <div class="my-3">
            <span class="mb-2 font-bold">"Changes"</span>
            {move || {
                grouped_by_lines()
                    .into_iter()
                    .map(|line| {
                        view! { cx,
                            <p>
                                {move || {
                                    line.iter()
                                        .map(|(text, color)| {
                                            view! { cx, <span style=color.styles()>{text}</span> }
                                        })
                                    .collect::<Vec<_>>()
                                }}
                            </p>
                        }
                    })
                    .collect::<Vec<_>>()
            }}
        </div>
    }
} -->
