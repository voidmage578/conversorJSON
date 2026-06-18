// src/components/DropZone.tsx

import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { FilePreview } from "../types/file";
import { SUPPORTED_EXTENSIONS } from "../types/file";

interface DropZoneProps {
  onFileLoaded: (preview: FilePreview, filePath: string) => void;
}

export function DropZone({ onFileLoaded }: DropZoneProps) {
  const [isDragging, setIsDragging] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Função central que chama o comando Rust e trata o resultado.
  // Tanto o drag-and-drop como o botão "Procurar" usam esta mesma função,
  // para nunca termos lógica de validação duplicada.
  const loadFile = useCallback(
    async (path: string) => {
      setError(null);

      // Validação simples da extensão antes de chamar o Rust —
      // dá feedback mais rápido ao utilizador sem esperar pelo backend.
      const extension = path.split(".").pop()?.toLowerCase();
      if (!extension || !SUPPORTED_EXTENSIONS.includes(extension as any)) {
        setError(
          `Formato não suportado. Usa um ficheiro .${SUPPORTED_EXTENSIONS.join(", .")}`
        );
        return;
      }

      setIsLoading(true);
      try {
        const preview = await invoke<FilePreview>("read_file_preview", { path });
        onFileLoaded(preview, path);
      } catch (err) {
        setError(typeof err === "string" ? err : "Erro desconhecido ao ler o ficheiro.");
      } finally {
        setIsLoading(false);
      }
    },
    [onFileLoaded]
  );

  // Regista o listener nativo de drag-and-drop da janela Tauri.
  // Isto substitui completamente os eventos onDragOver/onDrop do HTML5.
  useEffect(() => {
    const webview = getCurrentWebview();

    // onDragDropEvent devolve uma função de "cleanup" (unlisten) que
    // chamamos quando o componente desmonta, para não deixar listeners pendurados.
    const unlistenPromise = webview.onDragDropEvent((event) => {
      switch (event.payload.type) {
        case "enter":
        case "over":
          setIsDragging(true);
          break;
        case "drop":
          setIsDragging(false);
          // event.payload.paths é um array de caminhos (suporta múltiplos
          // ficheiros largados ao mesmo tempo) — pegamos só no primeiro.
          if (event.payload.paths.length > 0) {
            loadFile(event.payload.paths[0]);
          }
          break;
        case "leave":
          setIsDragging(false);
          break;
      }
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [loadFile]);

  const handleBrowseClick = useCallback(async () => {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "Folhas de cálculo",
          extensions: SUPPORTED_EXTENSIONS,
        },
      ],
    });

    if (selected && typeof selected === "string") {
      await loadFile(selected);
    }
  }, [loadFile]);

  return (
    <div className="w-full max-w-xl mx-auto">
      <div
        className={`
          border-2 border-dashed rounded-xl p-10 text-center transition-colors
          ${isDragging ? "border-blue-500 bg-blue-50" : "border-slate-300 bg-white"}
        `}
      >
        <p className="text-slate-600 mb-4">
          Arrasta um ficheiro .xlsx, .xls ou .csv para aqui
        </p>
        <p className="text-slate-400 text-sm mb-4">ou</p>
        <button
          onClick={handleBrowseClick}
          disabled={isLoading}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 transition-colors"
        >
          {isLoading ? "A carregar..." : "Procurar ficheiro..."}
        </button>
      </div>

      {error && (
        <p className="mt-3 text-red-600 text-sm text-center">{error}</p>
      )}
    </div>
  );
}