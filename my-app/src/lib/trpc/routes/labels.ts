import { t } from "../t";
import type { YakManLabelType } from "$lib/types/types";
import { YakManLabelTypeSchema } from "$lib/types/types";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";

const BASE_URL = getYakManBaseApiUrl();

export const fetchLabels = t.procedure
    .query(async ({ ctx }): Promise<YakManLabelType[]> => {
        const response = await fetch(`${BASE_URL}/v1/labels`, {
            headers: createYakManAuthHeaders(ctx.accessToken)
        });

        return await response.json();
    });

export const createLabel = t.procedure
    .input(YakManLabelTypeSchema)
    .mutation(async ({ input, ctx }) => {
        const response = await fetch(`${BASE_URL}/v1/labels`, {
            method: 'PUT',
            headers: {
                ...createYakManAuthHeaders(ctx.accessToken),
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(input)
        });
        if (response.status != 200) {
            throw new Error(await response.text())
        }
    });

