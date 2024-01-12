export function hasRoles(globalRoles: string[], roles: { [key: string]: string }) {
    return globalRoles.length > 0 || Object.keys(roles).length > 0;
}