import type { Cookies } from "@sveltejs/kit";
import parse from "set-cookie-parser";

/**
 * A utility for copying cookies from the YakMan Backend API to the cookies in the browser
 */
export function copyCookiesFromResponse(backendResponse: Response, cookies: Cookies) {
    for (const cookie of parse(backendResponse as any)) {
        cookies.set(cookie.name, cookie.value, {
            httpOnly: cookie.httpOnly,
            path: cookie.path,
            maxAge: cookie.maxAge,
        });
    }
}