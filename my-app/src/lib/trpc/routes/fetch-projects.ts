import { t } from "../t";
import type { YakManProject } from "$lib/types/types";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";

const BASE_URL = getYakManBaseApiUrl();

export const fetchProjects = t.procedure
    .query(async ({ ctx }): Promise<YakManProject[]> => {
        const response = await fetch(`${BASE_URL}/v1/projects`, {
            headers: createYakManAuthHeaders(ctx.accessToken)
        });
        return await response.json();
    });
