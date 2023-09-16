<script lang="ts">
  import type monaco from "monaco-editor";
  import { onDestroy, onMount } from "svelte";
  import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
  import jsonWorker from "monaco-editor/esm/vs/language/json/json.worker?worker";
  import cssWorker from "monaco-editor/esm/vs/language/css/css.worker?worker";
  import htmlWorker from "monaco-editor/esm/vs/language/html/html.worker?worker";
  import tsWorker from "monaco-editor/esm/vs/language/typescript/ts.worker?worker";
  import type { MonacoLanguage } from "$lib/utils/content-type-utils";

  let divEl: HTMLDivElement | null = null;
  let editor: monaco.editor.IStandaloneCodeEditor;
  let Monaco;

  export let content = "";
  export let language: MonacoLanguage = "text";

  onMount(async () => {
    self.MonacoEnvironment = {
      getWorker: function (_moduleId: string, label: string) {
        if (label === "json") {
          return new jsonWorker();
        }
        if (label === "css" || label === "scss" || label === "less") {
          return new cssWorker();
        }
        if (label === "html" || label === "handlebars" || label === "razor") {
          return new htmlWorker();
        }
        if (label === "typescript" || label === "javascript") {
          return new tsWorker();
        }
        return new editorWorker(); // plain text
      },
    };

    Monaco = await import("monaco-editor");
    editor = Monaco.editor.create(divEl!, {
      value: content,
      language: language,
    });

    editor.onDidChangeModelContent(() => (content = editor.getValue()));
  });

  $: {
    // @ts-ignore setLanguage exists but is not on type def for some reason
    editor?.getModel().setLanguage(language);
  }

  onDestroy(() => {
    editor?.dispose();
  });
</script>

<div class="h-full w-full shadow-sm rounded-md border border-gray-300">
  <div bind:this={divEl} class="h-full w-full" />
</div>

<!-- class="bg-white hover:bg-gray-50 disabled:bg-gray-300 rounded-md shadow-sm py-1 px-4 m-1 text-lg text-gray-700 border border-gray-300 transition-colors duration-200" -->
