import { fetchConfigs, fetchProjects, fetchConfigMetadata } from "$lib/api";
import type { PageServerLoad } from "./$types";
// import type { PageLoad } from "./$types";

// since there's no dynamic data here, we can prerender
// it so that it gets served as a static asset in production
export const prerender = true;

export const load: PageServerLoad = async ({ fetch, url }) => {
    const projects = await fetchProjects(fetch);

    const projectUuidQueryParam = url.searchParams.get('project');

    const selectedProject = !!projectUuidQueryParam ? projects.find(p => p.uuid === projectUuidQueryParam) : projects[0];

    const configs = await fetchConfigs(fetch, selectedProject?.uuid);

    const formattedConfigs = [];

    for (const config of configs) {
        const metadata = await fetchConfigMetadata(fetch, config.name);
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

