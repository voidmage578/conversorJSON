// src/types/mapping.ts

export type ColumnType = "text" | "number" | "boolean" | "date";

export type EmptyCellStrategy = "omit" | "null" | "default";

export interface ColumnConfig {
  /** O nome do cabeçalho tal como veio do ficheiro (ex: "Nome Cliente") */
  header: string;
  /** Tipo de dados forçado para esta coluna */
  type: ColumnType;
  /** Como tratar células vazias nesta coluna especificamente */
  emptyStrategy: EmptyCellStrategy;
}

/** Configuração completa de mapeamento: uma entrada por coluna do ficheiro */
export type MappingConfig = ColumnConfig[];