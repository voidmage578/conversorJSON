// src/types/file.ts

export interface FilePreview {
  headers: string[];
  sample_rows: string[][];
  total_rows_estimate: number;
}

export type SupportedExtension = "xlsx" | "xls" | "csv";

export const SUPPORTED_EXTENSIONS: SupportedExtension[] = ["xlsx", "xls", "csv"];