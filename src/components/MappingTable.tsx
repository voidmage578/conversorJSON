// src/components/MappingTable.tsx

import type { MappingConfig, ColumnType, EmptyCellStrategy } from "../types/mapping";

interface MappingTableProps {
  config: MappingConfig;
  sampleRows: string[][];
  headers: string[];
  onConfigChange: (newConfig: MappingConfig) => void;
}

const TYPE_LABELS: Record<ColumnType, string> = {
  text: "Texto",
  number: "Número",
  boolean: "Booleano",
  date: "Data",
};

const EMPTY_STRATEGY_LABELS: Record<EmptyCellStrategy, string> = {
  omit: "Omitir chave",
  null: "Definir como null",
  default: "Valor por defeito",
};

export function MappingTable({
  config,
  sampleRows,
  headers,
  onConfigChange,
}: MappingTableProps) {
  // Atualiza apenas um campo de uma coluna específica, mantendo as restantes intactas.
  const updateColumn = (
    columnIndex: number,
    changes: Partial<{ type: ColumnType; emptyStrategy: EmptyCellStrategy }>
  ) => {
    const newConfig = config.map((col, i) =>
      i === columnIndex ? { ...col, ...changes } : col
    );
    onConfigChange(newConfig);
  };

  // Para cada coluna, mostramos até 3 valores de amostra como contexto visual
  // (ex: "Ana, Bruno, Carla..."), para o utilizador confirmar que o tipo faz sentido.
  const getSamplePreviewText = (columnIndex: number): string => {
    const values = sampleRows
      .slice(0, 3)
      .map((row) => row[columnIndex] ?? "")
      .filter((v) => v !== "");
    return values.length > 0 ? values.join(", ") : "(sem dados de amostra)";
  };

  return (
    <div className="max-w-4xl mx-auto mt-8 bg-white rounded-lg shadow overflow-hidden">
      <table className="w-full text-sm">
        <thead className="bg-slate-100 text-slate-600 text-left">
          <tr>
            <th className="px-4 py-3 font-medium">Coluna</th>
            <th className="px-4 py-3 font-medium">Amostra</th>
            <th className="px-4 py-3 font-medium">Tipo</th>
            <th className="px-4 py-3 font-medium">Células vazias</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-100">
          {headers.map((header, columnIndex) => {
            const columnConfig = config[columnIndex];
            return (
              <tr key={header} className="hover:bg-slate-50">
                <td className="px-4 py-3 font-medium text-slate-800">
                  {header}
                </td>
                <td className="px-4 py-3 text-slate-500 max-w-xs truncate">
                  {getSamplePreviewText(columnIndex)}
                </td>
                <td className="px-4 py-3">
                  <select
                    value={columnConfig.type}
                    onChange={(e) =>
                      updateColumn(columnIndex, {
                        type: e.target.value as ColumnType,
                      })
                    }
                    className="border border-slate-300 rounded px-2 py-1 bg-white"
                  >
                    {Object.entries(TYPE_LABELS).map(([value, label]) => (
                      <option key={value} value={value}>
                        {label}
                      </option>
                    ))}
                  </select>
                </td>
                <td className="px-4 py-3">
                  <select
                    value={columnConfig.emptyStrategy}
                    onChange={(e) =>
                      updateColumn(columnIndex, {
                        emptyStrategy: e.target.value as EmptyCellStrategy,
                      })
                    }
                    className="border border-slate-300 rounded px-2 py-1 bg-white"
                  >
                    {Object.entries(EMPTY_STRATEGY_LABELS).map(([value, label]) => (
                      <option key={value} value={value}>
                        {label}
                      </option>
                    ))}
                  </select>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}