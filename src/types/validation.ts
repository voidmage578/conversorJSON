// src/types/validation.ts

export type ErrorPolicy = "strict" | "tolerant";

export interface ValidationError {
  row_number: number;
  column: string;
  raw_value: string;
  reason: string;
}

export interface ValidationReport {
  total_rows: number;
  valid_rows: number;
  invalid_rows: number;
  errors: ValidationError[];
  stopped_early: boolean;
}

export interface ExportResponse {
  report: ValidationReport;
  saved_path: string | null;
}