import { t } from "../t";
import type { YakManLabelType } from "$lib/types/types";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";

const BASE_URL = getYakManBaseApiUrl();

export const fetchLabels = t.procedure
    .query(async ({ ctx }): Promise<YakManLabelType[]> => {
        const response = await fetch(`${BASE_URL}/v1/labels`, {
            headers: createYakManAuthHeaders(ctx.accessToken)
        });

        return await response.json();
    });

