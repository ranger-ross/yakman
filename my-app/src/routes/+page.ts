import { trpc } from "$lib/trpc/client";
import type { PageLoad } from "./$types";

export const load: PageLoad = async (event) => {
    const projects = await trpc(event).projects.fetchProjects.query();

    const projectUuidQueryParam = event.url.searchParams.get('project');

    const selectedProject = !!projectUuidQueryParam ? projects.find(p => p.uuid === projectUuidQueryParam) : projects[0];

    const configs = await trpc(event).configs.fetchConfigs.query(selectedProject?.uuid);

    const formattedConfigs = [];

    for (const config of configs) {
        const metadata = await trpc(event).instances.fetchConfigMetadata.query(config.name);
        formattedConfigs.push({
            config: config,
            metadata: metadata
        });
    }

    return {
        projects: projects,
        configs: formattedConfigs
    };
}

