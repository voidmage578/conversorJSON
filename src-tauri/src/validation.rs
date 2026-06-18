// src-tauri/src/validation.rs

use serde::{Deserialize, Serialize};

/// Espelha o ColumnType do TypeScript (src/types/mapping.ts).
/// O #[derive(Deserialize)] permite ao Rust receber isto vindo do React em JSON.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColumnType {
    Text,
    Number,
    Boolean,
    Date,
}

/// Espelha o EmptyCellStrategy do TypeScript.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmptyCellStrategy {
    Omit,
    Null,
    Default,
}

/// Espelha o ColumnConfig do TypeScript — uma entrada por coluna do ficheiro.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")] 
pub struct ColumnConfig {
    pub header: String,
    #[serde(rename = "type")]
    pub column_type: ColumnType,
    #[allow(dead_code)]
    pub empty_strategy: EmptyCellStrategy,
}

/// A política de erro escolhida pelo utilizador antes de validar.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorPolicy {
    Strict,
    Tolerant,
}

/// Um único erro encontrado durante a validação, com localização exata.
#[derive(Debug, Serialize)]
pub struct ValidationError {
    /// Número da linha no ficheiro original (1-indexed, contando a partir
    /// da primeira linha de dados, não dos cabeçalhos).
    pub row_number: usize,
    pub column: String,
    pub raw_value: String,
    pub reason: String,
}

/// O relatório final devolvido ao frontend depois de validar o ficheiro completo.
#[derive(Debug, Serialize)]
pub struct ValidationReport {
    pub total_rows: usize,
    pub valid_rows: usize,
    pub invalid_rows: usize,
    pub errors: Vec<ValidationError>,
    /// Verdadeiro se o Modo Estrito tiver interrompido a validação a meio
    /// (o relatório fica parcial, porque paramos no primeiro erro).
    pub stopped_early: bool,
}

/// Resultado de uma tentativa de parsing — equivalente ao ParseResult do TS.
/// Usamos um enum porque em Rust é idiomático representar "ou sucesso ou erro"
/// desta forma, em vez de campos booleanos soltos.
#[allow(dead_code)]
pub enum ParsedValue {
    Text(String),
    Number(f64),
    Boolean(bool),
    /// Data já normalizada para ISO 8601 (AAAA-MM-DD).
    Date(String),
}

/// Tenta interpretar um valor em bruto de acordo com o tipo forçado.
/// Devolve Ok(valor) em caso de sucesso, ou Err(motivo) em caso de falha —
/// este é o uso idiomático de Result<T, E> que mencionei antes: a função
/// "pode falhar" e isso fica explícito na assinatura.
pub fn parse_value_as_type(raw_value: &str, column_type: &ColumnType) -> Result<ParsedValue, String> {
    // O trim() remove espaços invisíveis no início/fim — isto resolve
    // o requisito de "Sanitização" da Fase C ao mesmo tempo que fazemos parsing.
    let trimmed = raw_value.trim();

    match column_type {
        ColumnType::Text => Ok(ParsedValue::Text(trimmed.to_string())),
        ColumnType::Number => parse_as_number(trimmed),
        ColumnType::Boolean => parse_as_boolean(trimmed),
        ColumnType::Date => parse_as_date(trimmed),
    }
}

fn parse_as_number(value: &str) -> Result<ParsedValue, String> {
    // Aceita vírgula decimal (formato PT) substituindo por ponto antes de converter.
    let normalized = value.replace(',', ".");
    normalized
        .parse::<f64>()
        .map(ParsedValue::Number)
        .map_err(|_| format!("\"{}\" não é um número válido", value))
}

fn parse_as_boolean(value: &str) -> Result<ParsedValue, String> {
    let normalized = value.to_lowercase();
    match normalized.as_str() {
        "true" | "verdadeiro" | "sim" | "1" => Ok(ParsedValue::Boolean(true)),
        "false" | "falso" | "não" | "nao" | "0" => Ok(ParsedValue::Boolean(false)),
        _ => Err(format!("\"{}\" não é um valor booleano reconhecido", value)),
    }
}

fn parse_as_date(value: &str) -> Result<ParsedValue, String> {
    // Caso 1: número serial do Excel.
    if let Ok(serial) = value.parse::<f64>() {
        if let Some(date_str) = excel_serial_to_iso(serial) {
            return Ok(ParsedValue::Date(date_str));
        }
    }

    // Caso 2: ISO 8601 (AAAA-MM-DD).
    if let Some(date_str) = try_parse_iso_date(value) {
        return Ok(ParsedValue::Date(date_str));
    }

    // Caso 3: DD/MM/AAAA ou DD-MM-AAAA.
    if let Some(date_str) = try_parse_european_date(value) {
        return Ok(ParsedValue::Date(date_str));
    }

    Err(format!(
        "\"{}\" não corresponde a nenhum formato de data reconhecido",
        value
    ))
}

/// Converte um número serial do Excel (dias desde 1899-12-30) para "AAAA-MM-DD".
/// Usamos a crate `chrono` para fazer a aritmética de datas com segurança.
fn excel_serial_to_iso(serial: f64) -> Option<String> {
    use chrono::{Duration, NaiveDate};
    let excel_epoch = NaiveDate::from_ymd_opt(1899, 12, 30)?;
    let date = excel_epoch + Duration::days(serial as i64);
    Some(date.format("%Y-%m-%d").to_string())
}

fn try_parse_iso_date(value: &str) -> Option<String> {
    use chrono::NaiveDate;
    // Tentamos os primeiros 10 caracteres (caso venha com hora colada, ex: "T00:00:00").
    let candidate = value.get(0..10).unwrap_or(value);
    NaiveDate::parse_from_str(candidate, "%Y-%m-%d")
        .ok()
        .map(|d| d.format("%Y-%m-%d").to_string())
}

fn try_parse_european_date(value: &str) -> Option<String> {
    use chrono::NaiveDate;
    // Tentamos primeiro com "/" e depois com "-" como separador.
    let formats = ["%d/%m/%Y", "%d-%m-%Y"];
    for fmt in formats {
        if let Ok(date) = NaiveDate::parse_from_str(value, fmt) {
            return Some(date.format("%Y-%m-%d").to_string());
        }
    }
    None
}

use calamine::{open_workbook_auto, Reader};
use std::path::Path;

/// Comando principal exposto ao frontend. Recebe o caminho do ficheiro,
/// a configuração de mapeamento (uma entrada por coluna) e a política de erro,
/// e devolve um relatório completo de validação.
#[tauri::command]
pub fn validate_file(
    path: String,
    mapping: Vec<ColumnConfig>,
    policy: ErrorPolicy,
) -> Result<ValidationReport, String> {
    let path_obj = Path::new(&path);
    let extension = path_obj
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "xlsx" | "xls" => validate_excel(&path, &mapping, &policy),
        "csv" => validate_csv(&path, &mapping, &policy),
        _ => Err(format!("Formato de ficheiro não suportado: .{}", extension)),
    }
}

/// Valida uma única linha de dados contra a configuração de mapeamento.
/// Devolve a lista de erros encontrados nessa linha (vazia se a linha for válida).
/// Esta função é partilhada entre o validador de Excel e o de CSV, para
/// nunca termos duas implementações da mesma regra de negócio.
fn validate_row(
    row_number: usize,
    row_values: &[String],
    mapping: &[ColumnConfig],
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    for (column_index, column_config) in mapping.iter().enumerate() {
        // get(column_index) em vez de indexação direta [column_index] —
        // evita que o programa rebente (panic) se a linha tiver menos
        // colunas que o cabeçalho (linhas "curtas", comuns em CSVs sujos).
        let raw_value = row_values.get(column_index).map(|s| s.as_str()).unwrap_or("");
        let trimmed = raw_value.trim();

        if trimmed.is_empty() {
            // Célula vazia: aplicamos a regra de emptyStrategy, não o parser de tipo.
            // "Omit" e "Null" nunca geram erro de validação — são escolhas válidas
            // do utilizador. "Default" também nunca falha, porque o valor por
            // defeito é sempre válido para o tipo (ex: 0 para número, "" para texto).
            continue;
        }

        // Célula com conteúdo: tentamos interpretar de acordo com o tipo forçado.
        if let Err(reason) = parse_value_as_type(trimmed, &column_config.column_type) {
            errors.push(ValidationError {
                row_number,
                column: column_config.header.clone(),
                raw_value: raw_value.to_string(),
                reason,
            });
        }
    }

    errors
}

fn validate_excel(
    path: &str,
    mapping: &[ColumnConfig],
    policy: &ErrorPolicy,
) -> Result<ValidationReport, String> {
    let mut workbook = open_workbook_auto(path)
        .map_err(|e| format!("Não foi possível abrir o ficheiro Excel: {}", e))?;

    let sheet_names = workbook.sheet_names().to_owned();
    let first_sheet_name = sheet_names
        .first()
        .ok_or_else(|| "O ficheiro Excel não contém nenhuma folha.".to_string())?;

    let range = workbook
        .worksheet_range(first_sheet_name)
        .map_err(|e| format!("Erro ao ler a folha '{}': {}", first_sheet_name, e))?;

    let mut rows_iter = range.rows();
    rows_iter.next(); // salta a linha de cabeçalhos, já processada na Fase A/B

    let mut report = ValidationReport {
        total_rows: 0,
        valid_rows: 0,
        invalid_rows: 0,
        errors: Vec::new(),
        stopped_early: false,
    };

    for (index, row) in rows_iter.enumerate() {
        let row_number = index + 1; // 1-indexed, conta a partir da primeira linha de dados
        let row_values: Vec<String> = row.iter().map(|cell| cell.to_string()).collect();

        let row_errors = validate_row(row_number, &row_values, mapping);
        report.total_rows += 1;

        if row_errors.is_empty() {
            report.valid_rows += 1;
        } else {
            report.invalid_rows += 1;
            report.errors.extend(row_errors);

            if matches!(policy, ErrorPolicy::Strict) {
                report.stopped_early = true;
                break;
            }
        }
    }

    Ok(report)
}

fn validate_csv(
    path: &str,
    mapping: &[ColumnConfig],
    policy: &ErrorPolicy,
) -> Result<ValidationReport, String> {
    let mut reader = csv::Reader::from_path(path)
        .map_err(|e| format!("Não foi possível abrir o ficheiro CSV: {}", e))?;

    let mut report = ValidationReport {
        total_rows: 0,
        valid_rows: 0,
        invalid_rows: 0,
        errors: Vec::new(),
        stopped_early: false,
    };

    // reader.records() faz streaming linha a linha — nunca carrega o
    // ficheiro completo para memória, mesmo em ficheiros com milhões de linhas.
    for (index, result) in reader.records().enumerate() {
        let record = result.map_err(|e| format!("Erro ao ler linha {} do CSV: {}", index + 1, e))?;
        let row_number = index + 1;
        let row_values: Vec<String> = record.iter().map(|field| field.to_string()).collect();

        let row_errors = validate_row(row_number, &row_values, mapping);
        report.total_rows += 1;

        if row_errors.is_empty() {
            report.valid_rows += 1;
        } else {
            report.invalid_rows += 1;
            report.errors.extend(row_errors);

            if matches!(policy, ErrorPolicy::Strict) {
                report.stopped_early = true;
                break;
            }
        }
    }

    Ok(report)
}

// src-tauri/src/validation.rs
// (adicionar a seguir às funções de parsing já existentes)

use serde_json::Value as JsonValue;

/// Converte um ParsedValue (resultado de parsing bem-sucedido) no
/// equivalente serde_json::Value, para podermos construir o objeto final.
fn parsed_value_to_json(parsed: ParsedValue) -> JsonValue {
    match parsed {
        ParsedValue::Text(s) => JsonValue::String(s),
        ParsedValue::Number(n) => {
            // serde_json::Number não aceita NaN/infinito, por isso tratamos
            // com cuidado — mas como já validámos o parsing antes, isto
            // não deve acontecer na prática.
            serde_json::Number::from_f64(n)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null)
        }
        ParsedValue::Boolean(b) => JsonValue::Bool(b),
        ParsedValue::Date(s) => JsonValue::String(s),
    }
}

/// Decide o valor JSON de uma célula vazia, de acordo com a estratégia
/// escolhida pelo utilizador para essa coluna. Devolve None quando a
/// estratégia é "Omit" — sinal para quem chama esta função de que a
/// chave deve ser completamente excluída do objeto.
fn empty_cell_value(strategy: &EmptyCellStrategy, column_type: &ColumnType) -> Option<JsonValue> {
    match strategy {
        EmptyCellStrategy::Omit => None,
        EmptyCellStrategy::Null => Some(JsonValue::Null),
        EmptyCellStrategy::Default => Some(match column_type {
            ColumnType::Text | ColumnType::Date => JsonValue::String(String::new()),
            ColumnType::Number => JsonValue::Number(serde_json::Number::from(0)),
            ColumnType::Boolean => JsonValue::Bool(false),
        }),
    }
}

/// Constrói o objeto JSON de uma linha já validada (sem erros).
/// Assume que a linha passou por validate_row sem problemas — chamar
/// isto numa linha com erro produziria resultados inconsistentes,
/// por isso esta função é sempre usada a par da validação, nunca isolada.
fn build_row_object(row_values: &[String], mapping: &[ColumnConfig]) -> JsonValue {
    let mut map = serde_json::Map::new();

    for (column_index, column_config) in mapping.iter().enumerate() {
        let raw_value = row_values.get(column_index).map(|s| s.as_str()).unwrap_or("");
        let trimmed = raw_value.trim();

        if trimmed.is_empty() {
            match empty_cell_value(&column_config.empty_strategy, &column_config.column_type) {
                Some(value) => {
                    map.insert(column_config.header.clone(), value);
                }
                None => {
                    // Estratégia "Omit": não inserimos a chave de todo.
                }
            }
            continue;
        }

        // Já sabemos que isto tem sucesso, porque só chamamos build_row_object
        // depois de confirmar que a linha não tem erros de validação.
        if let Ok(parsed) = parse_value_as_type(trimmed, &column_config.column_type) {
            map.insert(column_config.header.clone(), parsed_value_to_json(parsed));
        }
    }

    JsonValue::Object(map)
}

/// Resultado combinado de processar o ficheiro completo para exportação:
/// inclui o relatório de validação E o array de objetos já construído,
/// prontos a serializar para o ficheiro .json final.
pub struct ExportResult {
    pub report: ValidationReport,
    pub rows: Vec<JsonValue>,
}

/// Percorre o ficheiro uma única vez, validando e construindo o JSON
/// das linhas válidas ao mesmo tempo. Reaproveita validate_row para
/// decidir se a linha tem erro, e build_row_object para gerar o objeto
/// quando a linha está limpa.
fn process_file_for_export(
    path: &str,
    mapping: &[ColumnConfig],
    policy: &ErrorPolicy,
) -> Result<ExportResult, String> {
    let path_obj = Path::new(path);
    let extension = path_obj
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut report = ValidationReport {
        total_rows: 0,
        valid_rows: 0,
        invalid_rows: 0,
        errors: Vec::new(),
        stopped_early: false,
    };
    let mut rows: Vec<JsonValue> = Vec::new();

    // Função auxiliar local (closure) que processa uma linha já lida,
    // partilhada entre o ramo Excel e o ramo CSV abaixo — evita repetir
    // esta lógica duas vezes.
    let mut handle_row = |row_number: usize, row_values: Vec<String>| -> bool {
        let row_errors = validate_row(row_number, &row_values, mapping);
        report.total_rows += 1;

        if row_errors.is_empty() {
            report.valid_rows += 1;
            rows.push(build_row_object(&row_values, mapping));
            false // linha sem erro
        } else {
            report.invalid_rows += 1;
            report.errors.extend(row_errors);
            true // linha COM erro
        }
    };

    match extension.as_str() {
        "xlsx" | "xls" => {
            let mut workbook = open_workbook_auto(path)
                .map_err(|e| format!("Não foi possível abrir o ficheiro Excel: {}", e))?;
            let sheet_names = workbook.sheet_names().to_owned();
            let first_sheet_name = sheet_names
                .first()
                .ok_or_else(|| "O ficheiro Excel não contém nenhuma folha.".to_string())?;
            let range = workbook
                .worksheet_range(first_sheet_name)
                .map_err(|e| format!("Erro ao ler a folha '{}': {}", first_sheet_name, e))?;

            let mut rows_iter = range.rows();
            rows_iter.next(); // salta cabeçalhos

            for (index, row) in rows_iter.enumerate() {
                let row_values: Vec<String> = row.iter().map(|cell| cell.to_string()).collect();
                let had_error = handle_row(index + 1, row_values);

                if matches!(policy, ErrorPolicy::Strict) && had_error {
                    report.stopped_early = true;
                    break;
                }
            }
        }
        "csv" => {
            let mut reader = csv::Reader::from_path(path)
                .map_err(|e| format!("Não foi possível abrir o ficheiro CSV: {}", e))?;

            for (index, result) in reader.records().enumerate() {
                let record = result
                    .map_err(|e| format!("Erro ao ler linha {} do CSV: {}", index + 1, e))?;
                let row_values: Vec<String> = record.iter().map(|f| f.to_string()).collect();
                let had_error = handle_row(index + 1, row_values);

                if matches!(policy, ErrorPolicy::Strict) && had_error {
                    report.stopped_early = true;
                    break;
                }
            }
        }
        _ => return Err(format!("Formato de ficheiro não suportado: .{}", extension)),
    }

    Ok(ExportResult { report, rows })
}

use std::fs;
use tauri_plugin_dialog::DialogExt;

/// Resposta enviada ao frontend depois de uma tentativa de exportação.
/// Inclui sempre o relatório (mesmo que o utilizador cancele o diálogo,
/// para o frontend poder mostrar o que se passou), e o caminho onde
/// o ficheiro foi guardado, se a operação chegou a esse ponto.
#[derive(Serialize)]
pub struct ExportResponse {
    pub report: ValidationReport,
    /// None se o utilizador cancelou o diálogo "Guardar como"
    /// antes de qualquer ficheiro ser escrito.
    pub saved_path: Option<String>,
}

/// Comando exposto ao frontend: processa o ficheiro completo (valida +
/// constrói o JSON), abre o diálogo nativo "Guardar como", e escreve
/// o resultado formatado no disco.
#[tauri::command]
pub async fn export_to_json(
    app: tauri::AppHandle,
    path: String,
    mapping: Vec<ColumnConfig>,
    policy: ErrorPolicy,
) -> Result<ExportResponse, String> {
    // Passo 1: processa o ficheiro completo (mesma lógica que já tínhamos).
    let export_result = process_file_for_export(&path, &mapping, &policy)?;

    // Passo 2: abre o diálogo nativo "Guardar como" e espera a escolha do utilizador.
    // O diálogo do Tauri é assíncrono via callback; envolvemos isto num
    // canal (channel) para conseguirmos "esperar" pelo resultado com .await,
    // que é mais natural de ler do que lidar com callbacks aninhados.
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .add_filter("JSON", &["json"])
        .set_file_name("dados_exportados.json")
        .save_file(move |file_path| {
            // Ignoramos o erro de send aqui porque, se o canal já não
            // estiver à escuta (caso extremamente raro), não há nada a fazer.
            let _ = tx.send(file_path);
        });

    let chosen_path = rx
        .recv()
        .map_err(|_| "Erro interno ao aguardar a escolha do ficheiro.".to_string())?;

    let chosen_path = match chosen_path {
        Some(p) => p,
        None => {
            // Utilizador cancelou o diálogo — devolvemos o relatório
            // (já calculado) mas sem caminho guardado.
            return Ok(ExportResponse {
                report: export_result.report,
                saved_path: None,
            });
        }
    };

    // Passo 3: serializa o array de objetos para JSON formatado (pretty-print)
    // e escreve no caminho escolhido.
    let json_string = serde_json::to_string_pretty(&export_result.rows)
        .map_err(|e| format!("Erro ao formatar o JSON: {}", e))?;

    let path_string = chosen_path.to_string();
    fs::write(&path_string, json_string)
        .map_err(|e| format!("Erro ao escrever o ficheiro: {}", e))?;

    Ok(ExportResponse {
        report: export_result.report,
        saved_path: Some(path_string),
    })
}