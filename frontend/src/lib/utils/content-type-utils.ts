export type MonacoLanguage = "json" | "html" | "typescript" | "css" | "text";


export function contentTypeToMonacoLanguage(contentType?: string | null): MonacoLanguage {
    if (!contentType)
        return "text";
    switch (contentType) {
        case "application/json":
            return "json";
        case "text/html":
            return "html";
        default:
            return "text";
    }
}
