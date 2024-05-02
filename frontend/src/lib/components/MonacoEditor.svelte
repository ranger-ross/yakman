<script lang="ts">
  import type monaco from "monaco-editor";
  import { onDestroy, onMount, afterUpdate } from "svelte";
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
  export let disabled: boolean = false;

  onMount(async () => {
    // Load Monaco and Yaml plugin dynamically because they are client-side only libraries
    Monaco = await import("monaco-editor");
    const EditorWorker = await import(
      "monaco-editor/esm/vs/editor/editor.worker?worker"
    );
    const YamlWorker = await import("$lib/utils/yaml.worker?worker");
    const { configureMonacoYaml } = await import("monaco-yaml");

    self.MonacoEnvironment = {
      getWorker: function (_moduleId: string, label: string) {
        switch (label) {
          case "editorWorkerService":
            return new EditorWorker.default();
          case "yaml":
            return new YamlWorker.default();
          case "json":
            return new jsonWorker();
          case "javascript":
          case "typescript":
            return new tsWorker();
          case "html":
            return new htmlWorker();
          case "css":
            return new cssWorker();
          default:
            throw new editorWorker(); // plain text
        }
      },
    };

    configureMonacoYaml(Monaco, {
      enableSchemaRequest: true,
      schemas: [],
    });

    Monaco.editor.defineTheme("yakmanMonacoTheme", {
      base: "vs",
      inherit: true,
      colors: {
        "editor.background": "#ffffff00", // transparent
      },
      rules: [],
    });

    editor = Monaco.editor.create(divEl!, {
      value: content,
      language: language,
      lineDecorationsWidth: 0,
      lineNumbersMinChars: 2,
      minimap: { enabled: false },
      overviewRulerLanes: 0,
      renderLineHighlight: "gutter",
      theme: "yakmanMonacoTheme",
    });

    editor.onDidChangeModelContent(() => (content = editor.getValue()));
  });

  // When props get updated, we need to manually update the Monaco Editor
  afterUpdate(() => {
    if (content != editor?.getValue()) {
      editor?.setValue(content);
    }
  });

  $: {
    // If the editor is disabled (readonly mode) and the content changes update the editor value.
    // NOTE: If the editor value changes, it will reset the cursor position
    //       so it will be annoying if this is enabled if the editor is enabled.
    if (editor && disabled) {
      editor.setValue(content);
    }
  }

  $: {
    // @ts-ignore setLanguage exists but is not on type def for some reason
    editor?.getModel().setLanguage(language);
  }

  $: {
    editor?.updateOptions({ readOnly: disabled });
  }

  onDestroy(() => {
    editor?.dispose();
  });
</script>

<svelte:window
  on:resize={() => {
    window.requestAnimationFrame(() => {
      const rect = divEl?.parentElement?.getBoundingClientRect();
      if (rect) {
        editor.layout({ width: rect.width, height: rect.height });
      }
    });
  }}
/>

<div class="h-full shadow-sm rounded border border-gray-300">
  <div bind:this={divEl} class="h-full" />
</div>
