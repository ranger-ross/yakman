import { t } from "../t";
import { createYakManAuthHeaders, getYakManBaseApiUrl } from "../helper";
import { YakManLabelTypeSchema } from "$lib/types/types";

const BASE_URL = getYakManBaseApiUrl();

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
