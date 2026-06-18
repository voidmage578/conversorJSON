// src/App.tsx

import { useState } from "react";
import { DropZone } from "./components/DropZone";
import { MappingTable } from "./components/MappingTable";
import { ValidationPanel } from "./components/ValidationPanel";
import { buildInitialMappingConfig } from "./lib/inferType";
import type { FilePreview } from "./types/file";
import type { MappingConfig } from "./types/mapping";

function App() {
  const [preview, setPreview] = useState<FilePreview | null>(null);
  const [filePath, setFilePath] = useState<string | null>(null);
  const [mappingConfig, setMappingConfig] = useState<MappingConfig>([]);

  const handleFileLoaded = (loadedPreview: FilePreview, path: string) => {
    setPreview(loadedPreview);
    setFilePath(path);
    setMappingConfig(
      buildInitialMappingConfig(loadedPreview.headers, loadedPreview.sample_rows)
    );
  };

  // Repõe todo o estado, voltando ao ecrã inicial de carregamento.
  const handleReload = () => {
    setPreview(null);
    setFilePath(null);
    setMappingConfig([]);
  };

  return (
    <main className="min-h-screen bg-slate-50 p-8">
      <h1 className="text-2xl font-semibold text-slate-800 text-center mb-8">
        XLSX → JSON Converter
      </h1>

      {!preview && <DropZone onFileLoaded={handleFileLoaded} />}

      {preview && filePath && (
        <>
          <p className="text-center text-sm text-slate-500 mt-6">
            {filePath} — {preview.total_rows_estimate} linhas estimadas
          </p>
          <MappingTable
            config={mappingConfig}
            sampleRows={preview.sample_rows}
            headers={preview.headers}
            onConfigChange={setMappingConfig}
          />
          <ValidationPanel
            filePath={filePath}
            mappingConfig={mappingConfig}
            onReload={handleReload}
          />
        </>
      )}
    </main>
  );
}

export default App;