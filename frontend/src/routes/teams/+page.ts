import { trpc } from "$lib/trpc/client";
import type { PageLoad } from "./$types";

export const load: PageLoad = async (event) => {
    const teams = await trpc(event).teams.fetchTeams.query();
    return {
        teams
    };
}

