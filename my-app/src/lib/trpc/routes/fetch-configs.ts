import { z } from "zod";
import { t } from "../t";
import type { YakManConfig } from "$lib/types/types";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";

const BASE_URL = getYakManBaseApiUrl();

export const fetchConfigs = t.procedure
    .input(z.string().optional())
    .query(async ({ input, ctx }): Promise<YakManConfig[]> => {
        const response = await fetch(`${BASE_URL}/v1/configs` + (input ? `?project=${input}` : ``), {
            headers: createYakManAuthHeaders(ctx.accessToken)
        });
        return await response.json();
    });

