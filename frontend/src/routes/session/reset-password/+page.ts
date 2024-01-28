import type { PageLoad } from "../$types";


export const load: PageLoad = async (event) => {
    let id = event.url.searchParams.get('id');
    let userUuid = event.url.searchParams.get('user_uuid');
    return {
        id,
        userUuid,
    }
};