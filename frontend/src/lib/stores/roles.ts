import { writable } from "svelte/store";

type RoleState = {
    globalRoles: string[],
    roles: { [key: string]: string },
};

export const roles = writable(undefined as RoleState | undefined);

