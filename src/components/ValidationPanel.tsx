// src/components/ValidationPanel.tsx

import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { MappingConfig } from "../types/mapping";
import type { ErrorPolicy, ValidationReport, ExportResponse } from "../types/validation";

interface ValidationPanelProps {
  filePath: string;
  mappingConfig: MappingConfig;
  /** Chamado quando o utilizador escolhe "Recarregar ficheiro" — o
      App.tsx é responsável por repor todo o estado e mostrar o DropZone. */
  onReload: () => void;
}

export function ValidationPanel({ filePath, mappingConfig, onReload }: ValidationPanelProps) {
  const [policy, setPolicy] = useState<ErrorPolicy>("tolerant");
  const [isValidating, setIsValidating] = useState(false);
  const [isExporting, setIsExporting] = useState(false);
  const [report, setReport] = useState<ValidationReport | null>(null);
  const [savedPath, setSavedPath] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleValidate = useCallback(async () => {
    setError(null);
    setIsValidating(true);
    try {
      const result = await invoke<ValidationReport>("validate_file", {
        path: filePath,
        mapping: mappingConfig,
        policy,
      });
      setReport(result);
    } catch (err) {
      setError(typeof err === "string" ? err : "Erro inesperado durante a validação.");
    } finally {
      setIsValidating(false);
    }
  }, [filePath, mappingConfig, policy]);

  const handleExport = useCallback(async () => {
    setError(null);
    setIsExporting(true);
    try {
      const result = await invoke<ExportResponse>("export_to_json", {
        path: filePath,
        mapping: mappingConfig,
        policy,
      });
      setReport(result.report);
      setSavedPath(result.saved_path);
    } catch (err) {
      setError(typeof err === "string" ? err : "Erro inesperado durante a exportação.");
    } finally {
      setIsExporting(false);
    }
  }, [filePath, mappingConfig, policy]);

  const hasErrors = report ? report.invalid_rows > 0 : false;
  const canExport = report ? (policy === "tolerant" || !hasErrors) : false;

  return (
    <div className="max-w-4xl mx-auto mt-8 bg-white rounded-lg shadow p-6">
      <h2 className="text-lg font-semibold text-slate-800 mb-4">Validação</h2>

      {/* Ecrã 1: configuração inicial — só visível antes de validar */}
      {!report && (
        <>
          <div className="flex items-center gap-6 mb-4">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="policy"
                value="tolerant"
                checked={policy === "tolerant"}
                onChange={() => setPolicy("tolerant")}
              />
              <span className="text-sm text-slate-700">
                <strong>Modo Tolerante</strong> — ignora linhas com erro e gera relatório
              </span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="policy"
                value="strict"
                checked={policy === "strict"}
                onChange={() => setPolicy("strict")}
              />
              <span className="text-sm text-slate-700">
                <strong>Modo Estrito</strong> — cancela no primeiro erro
              </span>
            </label>
          </div>

          <button
            onClick={handleValidate}
            disabled={isValidating}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 transition-colors"
          >
            {isValidating ? "A validar..." : "Validar ficheiro"}
          </button>
        </>
      )}

      {error && <p className="mt-3 text-red-600 text-sm">{error}</p>}

      {/* Ecrã 2: relatório + ações, depois de validar */}
      {report && (
        <ValidationReportView
          report={report}
          policy={policy}
          canExport={canExport}
          isExporting={isExporting}
          savedPath={savedPath}
          onExport={handleExport}
          onReload={onReload}
        />
      )}
    </div>
  );
}

function ValidationReportView({
  report,
  policy,
  canExport,
  isExporting,
  savedPath,
  onExport,
  onReload,
}: {
  report: ValidationReport;
  policy: ErrorPolicy;
  canExport: boolean;
  isExporting: boolean;
  savedPath: string | null;
  onExport: () => void;
  onReload: () => void;
}) {
  const hasErrors = report.errors.length > 0;
  const firstError = report.errors[0];

  return (
    <div>
      <div className="grid grid-cols-3 gap-4 mb-4">
        <SummaryCard label="Total de linhas" value={report.total_rows} />
        <SummaryCard label="Linhas válidas" value={report.valid_rows} tone="green" />
        <SummaryCard label="Linhas com erro" value={report.invalid_rows} tone={hasErrors ? "red" : "green"} />
      </div>

      {/* Modo Estrito com erro: bloqueia exportação, mostra mensagem específica */}
      {policy === "strict" && hasErrors && firstError && (
        <p className="text-red-700 bg-red-50 border border-red-200 rounded p-3 text-sm mb-4">
          Exportação indisponível no Modo Estrito. Por favor, corrija o erro
          na linha {firstError.row_number} (coluna "{firstError.column}") ou recarregue
          o ficheiro e escolha o Modo Tolerante.
        </p>
      )}

      {/* Modo Tolerante com erro: aviso informativo, exportação permitida */}
      {policy === "tolerant" && hasErrors && (
        <p className="text-amber-700 bg-amber-50 border border-amber-200 rounded p-3 text-sm mb-4">
          {report.invalid_rows} linha(s) serão ignoradas por terem erros. As
          restantes {report.valid_rows} linhas válidas podem ser exportadas.
        </p>
      )}

      {!hasErrors && (
        <p className="text-green-700 bg-green-50 border border-green-200 rounded p-3 text-sm mb-4">
          Nenhum erro encontrado. O ficheiro está pronto para exportação.
        </p>
      )}

      {hasErrors && (
        <div className="overflow-hidden rounded-lg border border-slate-200 mb-4">
          <table className="w-full text-sm">
            <thead className="bg-slate-100 text-slate-600 text-left">
              <tr>
                <th className="px-4 py-2 font-medium">Linha</th>
                <th className="px-4 py-2 font-medium">Coluna</th>
                <th className="px-4 py-2 font-medium">Valor</th>
                <th className="px-4 py-2 font-medium">Motivo</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {report.errors.map((err, i) => (
                <tr key={i} className="hover:bg-slate-50">
                  <td className="px-4 py-2 text-slate-700">{err.row_number}</td>
                  <td className="px-4 py-2 text-slate-700">{err.column}</td>
                  <td className="px-4 py-2 text-slate-500 font-mono">{err.raw_value || "(vazio)"}</td>
                  <td className="px-4 py-2 text-red-600">{err.reason}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Confirmação de exportação já concluída */}
      {savedPath && (
        <p className="text-green-700 bg-green-50 border border-green-200 rounded p-3 text-sm mb-4">
          Ficheiro exportado com sucesso: {savedPath}
        </p>
      )}

      {/* Ações finais, de acordo com a matriz de comportamento definida */}
      <div className="flex gap-3">
        {canExport && !savedPath && (
          <button
            onClick={onExport}
            disabled={isExporting}
            className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:opacity-50 transition-colors"
          >
            {isExporting
              ? "A exportar..."
              : `Exportar JSON (${report.valid_rows} linhas válidas)`}
          </button>
        )}

        {(policy === "tolerant" && hasErrors) || (policy === "strict" && hasErrors) || savedPath ? (
          <button
            onClick={onReload}
            className="px-4 py-2 bg-slate-200 text-slate-700 rounded-lg hover:bg-slate-300 transition-colors"
          >
            Recarregar ficheiro
          </button>
        ) : null}
      </div>
    </div>
  );
}

function SummaryCard({
  label,
  value,
  tone = "default",
}: {
  label: string;
  value: number;
  tone?: "default" | "green" | "red";
}) {
  const toneClasses = {
    default: "bg-slate-50 text-slate-700",
    green: "bg-green-50 text-green-700",
    red: "bg-red-50 text-red-700",
  };
  return (
    <div className={`rounded-lg p-4 text-center ${toneClasses[tone]}`}>
      <p className="text-2xl font-semibold">{value}</p>
      <p className="text-xs mt-1">{label}</p>
    </div>
  );
}