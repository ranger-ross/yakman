export type MonacoLanguage = "json" | "html" | "typescript" | "css" | "text" | "yaml";


export function contentTypeToMonacoLanguage(contentType?: string | null): MonacoLanguage {
    if (!contentType)
        return "text";
    if (contentType.includes("yaml") || contentType.includes("yml")) {
        return "yaml";
    }
    switch (contentType) {
        case "application/json":
            return "json";
        case "text/html":
            return "html";
        default:
            return "text";
    }
}
