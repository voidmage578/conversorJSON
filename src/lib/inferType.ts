// src/lib/inferType.ts

import type { ColumnType } from "../types/mapping";
import { parseAsDate } from "./parseValue";

/**
 * Tenta adivinhar o tipo de uma coluna com base nos valores de amostra.
 * Estratégia: se TODOS os valores não-vazios da amostra encaixam num tipo
 * mais específico (número, booleano, data), usamos esse tipo.
 * Caso contrário, ou se a amostra estiver vazia, assumimos "text" (o tipo
 * mais seguro, porque qualquer valor é válido como texto).
 */
export function inferColumnType(sampleValues: string[]): ColumnType {
  // Ignoramos valores vazios na análise — uma célula em branco não
  // deve "contaminar" a inferência de tipo da coluna.
  const nonEmptyValues = sampleValues
    .map((v) => v.trim())
    .filter((v) => v.length > 0);

  if (nonEmptyValues.length === 0) {
    return "text";
  }

  if (nonEmptyValues.every(isLikelyBoolean)) {
    return "boolean";
  }

  if (nonEmptyValues.every(isLikelyNumber)) {
    return "number";
  }

  if (nonEmptyValues.every(isLikelyDate)) {
    return "date";
  }

  return "text";
}

function isLikelyBoolean(value: string): boolean {
  const normalized = value.toLowerCase();
  return ["true", "false", "verdadeiro", "falso", "sim", "não"].includes(normalized);
}

function isLikelyNumber(value: string): boolean {
  // Aceita números com ponto ou vírgula decimal (ex: "12.5" ou "12,5")
  // e não aceita strings vazias (Number("") seria 0, o que seria errado aqui).
  if (value === "") return false;
  const normalized = value.replace(",", ".");
  return !Number.isNaN(Number(normalized));
}

function isLikelyDate(value: string): boolean {
  if (isLikelyNumber(value)) {
    // Um número "puro" pode ser uma data serial do Excel — mas só o
    // consideramos como tal se também passar pelo parser de data;
    // caso contrário, é mais provável que seja mesmo um número normal.
    // Por simplicidade na fase de inferência (sugestão, não certeza),
    // deixamos números puros como candidatos a "number", não "date" —
    // o utilizador corrige manualmente se for mesmo uma data serial.
    return false;
  }
  const result = parseAsDate(value);
  return result.success;
}

/**
 * Constrói a configuração inicial de mapeamento para todas as colunas,
 * usando a amostra de dados da Fase A para inferir o tipo de cada uma.
 */
export function buildInitialMappingConfig(
  headers: string[],
  sampleRows: string[][]
): import("../types/mapping").MappingConfig {
  return headers.map((header, columnIndex) => {
    const columnValues = sampleRows.map((row) => row[columnIndex] ?? "");
    return {
      header,
      type: inferColumnType(columnValues),
      emptyStrategy: "omit" as const,
    };
  });
}