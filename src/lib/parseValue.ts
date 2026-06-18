// src/lib/parseValue.ts

import type { ColumnType } from "../types/mapping";

/**
 * Resultado de uma tentativa de parsing de um valor.
 * "success" indica se conseguimos interpretar o valor corretamente.
 */
export interface ParseResult {
  success: boolean;
  /** Valor já convertido para o tipo certo (string ISO, number, boolean...) */
  value: string | number | boolean | null;
  /** Mensagem de erro legível, só presente quando success é false */
  error?: string;
}

/**
 * Tenta interpretar um valor de acordo com o tipo forçado pelo utilizador.
 * Esta função NÃO trata de células vazias — isso é responsabilidade
 * de outra camada (a regra de emptyStrategy). Aqui assumimos que o
 * valor já chegou não-vazio.
 */
export function parseValueAsType(rawValue: string, type: ColumnType): ParseResult {
  switch (type) {
    case "text":
      return { success: true, value: rawValue };
    case "number":
      return parseAsNumber(rawValue);
    case "boolean":
      return parseAsBoolean(rawValue);
    case "date":
      return parseAsDate(rawValue);
  }
}

function parseAsNumber(rawValue: string): ParseResult {
  // Aceita tanto "12.5" como "12,5" (vírgula decimal, comum em PT).
  const normalized = rawValue.trim().replace(",", ".");
  const parsed = Number(normalized);
  if (Number.isNaN(parsed)) {
    return {
      success: false,
      value: null,
      error: `"${rawValue}" não é um número válido`,
    };
  }
  return { success: true, value: parsed };
}

function parseAsBoolean(rawValue: string): ParseResult {
  const normalized = rawValue.trim().toLowerCase();
  const truthy = ["true", "verdadeiro", "sim", "1"];
  const falsy = ["false", "falso", "não", "nao", "0"];

  if (truthy.includes(normalized)) return { success: true, value: true };
  if (falsy.includes(normalized)) return { success: true, value: false };

  return {
    success: false,
    value: null,
    error: `"${rawValue}" não é um valor booleano reconhecido`,
  };
}

/**
 * Tenta interpretar uma data testando vários formatos comuns, por ordem.
 * Devolve sempre, em caso de sucesso, uma string ISO 8601 (AAAA-MM-DD),
 * conforme pedido nos requisitos da Fase A ("formatar datas consistentemente").
 */
export function parseAsDate(rawValue: string): ParseResult {
  const trimmed = rawValue.trim();

  // Caso 1: número serial do Excel (ex: "45000").
  // O Excel conta dias desde 1899-12-30 (dia 0), com uma particularidade
  // histórica de um bug de bissexto que todas as bibliotecas replicam por
  // compatibilidade — por isso usamos a mesma época de referência.
  if (/^\d+(\.\d+)?$/.test(trimmed)) {
    const serial = Number(trimmed);
    const excelEpoch = new Date(Date.UTC(1899, 11, 30));
    const resultDate = new Date(excelEpoch.getTime() + serial * 24 * 60 * 60 * 1000);
    if (!Number.isNaN(resultDate.getTime())) {
      return { success: true, value: toISODateString(resultDate) };
    }
  }

  // Caso 2: ISO 8601 (AAAA-MM-DD), já no formato que queremos.
  const isoMatch = trimmed.match(/^(\d{4})-(\d{2})-(\d{2})/);
  if (isoMatch) {
    const [, year, month, day] = isoMatch;
    const candidate = new Date(Date.UTC(Number(year), Number(month) - 1, Number(day)));
    if (!Number.isNaN(candidate.getTime())) {
      return { success: true, value: toISODateString(candidate) };
    }
  }

  // Caso 3: DD/MM/AAAA ou DD-MM-AAAA (formato europeu/português).
  const europeanMatch = trimmed.match(/^(\d{1,2})[/-](\d{1,2})[/-](\d{4})$/);
  if (europeanMatch) {
    const [, day, month, year] = europeanMatch;
    const candidate = new Date(Date.UTC(Number(year), Number(month) - 1, Number(day)));
    // Confirmamos que o dia/mês não "saltaram" de forma estranha
    // (ex: "31/02/2024" não existe, e o JS corrigiria silenciosamente).
    if (
      !Number.isNaN(candidate.getTime()) &&
      candidate.getUTCDate() === Number(day) &&
      candidate.getUTCMonth() === Number(month) - 1
    ) {
      return { success: true, value: toISODateString(candidate) };
    }
  }

  return {
    success: false,
    value: null,
    error: `"${rawValue}" não corresponde a nenhum formato de data reconhecido`,
  };
}

function toISODateString(date: Date): string {
  return date.toISOString().split("T")[0]; // "AAAA-MM-DD"
}