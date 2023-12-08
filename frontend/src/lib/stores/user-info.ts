import { writable } from "svelte/store";

type UserInfoState = {
    profilePictureUrl: string | null,
};

export const userInfo = writable({
    profilePictureUrl: null
} as UserInfoState);

