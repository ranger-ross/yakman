import { fetchConfigs, fetchProjects, fetchConfigMetadata } from "$lib/api";
import type { PageServerLoad } from "./$types";

// since there's no dynamic data here, we can prerender
// it so that it gets served as a static asset in production
export const prerender = true;

export const load: PageServerLoad = async (params) => {
    const projects = await fetchProjects();

    const selectedProject = projects[0]; // TODO: Also handle query params


    const configs = await fetchConfigs(selectedProject?.uuid);

    const formattedConfigs = [];

    for (const config of configs) {
        const metadata = await fetchConfigMetadata(config.name);
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

