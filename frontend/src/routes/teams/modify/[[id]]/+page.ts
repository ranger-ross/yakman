
import { trpc } from "$lib/trpc/client";
import type { YakManTeamDetails } from "$lib/types/types";
import type { PageLoad } from "./$types";

export const load: PageLoad = async (event) => {
    const teamId = event.params.id;
    let team: YakManTeamDetails | null = null;

    if (teamId) {
        team = await trpc(event).teams.fetchTeamById.query(teamId);
    }

    const projects = await trpc(event).projects.fetchProjects.query();
    const users = await trpc(event).users.fetchUsers.query();

    return {
        team,
        projects,
        users
    };
}

