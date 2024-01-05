import { TRPCError } from "@trpc/server";
import type { TRPC_ERROR_CODE_KEY } from "@trpc/server/rpc";

const HTTP_TO_TPRC_CODE: { [key: number]: TRPC_ERROR_CODE_KEY } = {
    400: "BAD_REQUEST",
    401: "UNAUTHORIZED",
    403: "FORBIDDEN",
    404: "NOT_FOUND",
    500: "INTERNAL_SERVER_ERROR"
};

export function convertYakManErrorToTRPCError(response: string | object, status: number) {
    if (response instanceof String) {
        response = JSON.parse(response as string)
    }

    return new TRPCError({
        code: HTTP_TO_TPRC_CODE[status] ?? "INTERNAL_SERVER_ERROR",
        message: JSON.stringify(response)
    })
}