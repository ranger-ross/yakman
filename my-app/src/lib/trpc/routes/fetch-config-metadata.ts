import { z } from "zod";
import { t } from "../t";
import type { YakManConfigInstance } from "$lib/types/types";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";

const BASE_URL = getYakManBaseApiUrl();

export const fetchConfigMetadata = t.procedure
    .input(z.string())
    .query(async ({ input, ctx }): Promise<YakManConfigInstance[]> => {
        const response = await fetch(`${BASE_URL}/v1/configs/${input}/instances`, {
            headers: createYakManAuthHeaders(ctx.accessToken)
        });
        return await response.json();
    });

