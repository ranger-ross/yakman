
import { trpc } from "$lib/trpc/client";
import type { YakManTeamDetails } from "$lib/types/types";
import type { PageLoad } from "./$types";
import { TRPCClientError } from '@trpc/client';

export const load: PageLoad = async (event) => {
    const teamId = event.params.id;
    let team: YakManTeamDetails | null = null;

    if (teamId) {
        team = await trpc(event).teams.fetchTeamById.query(teamId);
    }

    return {
        team
    };
}

