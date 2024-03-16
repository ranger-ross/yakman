export type MonacoLanguage = "json" | "html" | "typescript" | "css" | "text" | "yaml";


export function contentTypeToMonacoLanguage(contentType?: string | null): MonacoLanguage {
    if (!contentType)
        return "text";
    if (contentType.toLocaleLowerCase().includes("yaml") ||
        contentType.toLocaleLowerCase().includes("yml")) {
        return "yaml";
    }
    if (contentType.toLocaleLowerCase().includes("json")) {
        return "json";
    }
    switch (contentType.toLocaleLowerCase()) {
        case "text/html":
            return "html";
        default:
            return "text";
    }
}
