import { createRouterCaller } from "$lib/trpc/router";
import type { PageServerLoad } from "./$types";

export const load: PageServerLoad = async (event) => {
    const trpc = await createRouterCaller(event);

    const projects = await trpc.projects.fetchProjects();

    const projectUuidQueryParam = event.url.searchParams.get('project');

    const selectedProject = !!projectUuidQueryParam ? projects.find(p => p.uuid === projectUuidQueryParam) : projects[0];

    const configs = await trpc.configs.fetchConfigs(selectedProject?.uuid);

    const formattedConfigs = [];

    for (const config of configs) {
        const metadata = await trpc.instances.fetchConfigMetadata(config.name);
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

