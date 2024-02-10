import { t } from "../t";
import { getYakManBaseApiUrl } from "../helper";
import { convertYakManErrorToTRPCError } from "$lib/utils/error-helpers";

const BASE_URL = getYakManBaseApiUrl();

type YakManApplicationSettings = {
    enable_oauth: boolean
}

export const lifecycle = t.router({
    fetchYakmanSettings: t.procedure
        .query(async (): Promise<YakManApplicationSettings> => {
            const response = await fetch(`${BASE_URL}/v1/settings`);
            if (response.status != 200) {
                throw convertYakManErrorToTRPCError(await response.text(), response.status)
            }
            return await response.json();
        }),
})


