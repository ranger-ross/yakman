import { trpc } from "$lib/trpc/client";
import type { PageLoad } from "./$types";
import type { YakManProject } from "$lib/types/types";
import { TRPCClientError } from '@trpc/client';

export const load: PageLoad = async (event) => {
    let projects: YakManProject[] = [];
    try {
        projects = await trpc(event).projects.fetchProjects.query();
    } catch (e) {
        // If non-internal server error, assume the token just needs to be refreshed.
        if (e instanceof TRPCClientError && e.data.code !== "INTERNAL_SERVER_ERROR") {
            projects = []
        } else {
            throw e;
        }
    }
    const projectIdQueryParam = event.url.searchParams.get('project');

    const selectedProject = !!projectIdQueryParam ? projects.find(p => p.id === projectIdQueryParam) : projects[0];

    const configs = (await trpc(event).configs.fetchConfigs.query(selectedProject?.id)) ?? [];
    const labels = await trpc(event).labels.fetchLabels.query();

    const formattedConfigs = [];

    for (const config of configs) {
        const instances = await trpc(event).instances.fetchInstancesByConfigId.query(config.id);
        formattedConfigs.push({
            config: config,
            instances: instances
        });
    }

    return {
        projects: projects,
        labels: labels,
        configs: formattedConfigs
    };

}

