import { t } from "../t";
import type { YakManProject } from "$lib/types/types";
import { getYakManBaseApiUrl } from "../helper";
import { convertYakManErrorToTRPCError } from "$lib/utils/error-helpers";

const BASE_URL = getYakManBaseApiUrl();

type YakManApplicationConfig = {
    enable_oauth: boolean
}

export const yakman = t.router({
    fetchYakmanConfig: t.procedure
        .query(async (): Promise<YakManApplicationConfig> => {
            const response = await fetch(`${BASE_URL}/yakman/config`);
            if (response.status != 200) {
                throw convertYakManErrorToTRPCError(await response.text(), response.status)
            }
            return await response.json();
        }),

})


