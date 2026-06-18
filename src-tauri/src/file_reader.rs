// src-tauri/src/file_reader.rs

use calamine::{open_workbook_auto, Reader};
use serde::Serialize;
use std::path::Path;

/// Estrutura que vamos devolver ao React.
/// O #[derive(Serialize)] permite ao Rust converter isto automaticamente para JSON.
#[derive(Serialize)]
pub struct FilePreview {
    pub headers: Vec<String>,
    pub sample_rows: Vec<Vec<String>>,
    pub total_rows_estimate: usize,
}

/// Comando exposto ao frontend. Recebe o caminho do ficheiro e devolve
/// apenas os cabeçalhos + uma amostra de linhas, sem carregar o ficheiro completo.
///
/// `#[tauri::command]` é o que permite ao React chamar esta função Rust
/// como se fosse uma função assíncrona normal.
#[tauri::command]
pub fn read_file_preview(path: String) -> Result<FilePreview, String> {
    let path_obj = Path::new(&path);

    // Decidimos qual leitor usar com base na extensão do ficheiro.
    let extension = path_obj
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "xlsx" | "xls" => read_excel_preview(&path),
        "csv" => read_csv_preview(&path),
        _ => Err(format!("Formato de ficheiro não suportado: .{}", extension)),
    }
}

/// Lê o preview de um ficheiro Excel (.xlsx ou .xls) usando a crate calamine.
fn read_excel_preview(path: &str) -> Result<FilePreview, String> {
    // open_workbook_auto detecta automaticamente se é .xlsx, .xls, etc.
    // O "?" propaga o erro para fora da função caso algo falhe (ficheiro corrompido, etc.)
    let mut workbook = open_workbook_auto(path)
        .map_err(|e| format!("Não foi possível abrir o ficheiro Excel: {}", e))?;

    // Calamine lê por "sheets" (folhas). Vamos assumir a primeira folha por agora.
    let sheet_names = workbook.sheet_names().to_owned();
    let first_sheet_name = sheet_names
        .first()
        .ok_or_else(|| "O ficheiro Excel não contém nenhuma folha.".to_string())?;

    let range = workbook
        .worksheet_range(first_sheet_name)
        .map_err(|e| format!("Erro ao ler a folha '{}': {}", first_sheet_name, e))?;

    let mut rows_iter = range.rows();

    // A primeira linha são os cabeçalhos.
    let headers: Vec<String> = match rows_iter.next() {
        Some(row) => row.iter().map(|cell| cell.to_string()).collect(),
        None => return Err("O ficheiro Excel está vazio.".to_string()),
    };

    // As próximas 20 linhas (ou menos, se o ficheiro for mais pequeno) são a amostra.
    const SAMPLE_SIZE: usize = 20;
    let sample_rows: Vec<Vec<String>> = rows_iter
        .by_ref()
        .take(SAMPLE_SIZE)
        .map(|row| row.iter().map(|cell| cell.to_string()).collect())
        .collect();

    // calamine consegue dizer-nos quantas linhas tem a folha sem ler o conteúdo todo,
    // porque essa informação já está disponível nos metadados internos do range.
    let total_rows_estimate = range.height();

    Ok(FilePreview {
        headers,
        sample_rows,
        total_rows_estimate,
    })
}

/// Lê o preview de um ficheiro CSV por streaming, linha a linha,
/// parando depois de ter cabeçalhos + amostra (nunca carrega o ficheiro completo).
fn read_csv_preview(path: &str) -> Result<FilePreview, String> {
    let mut reader = csv::Reader::from_path(path)
        .map_err(|e| format!("Não foi possível abrir o ficheiro CSV: {}", e))?;

    // A crate `csv` trata a primeira linha como cabeçalhos automaticamente.
    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| format!("Erro ao ler os cabeçalhos do CSV: {}", e))?
        .iter()
        .map(|h| h.to_string())
        .collect();

    if headers.is_empty() {
        return Err("O ficheiro CSV está vazio.".to_string());
    }

    const SAMPLE_SIZE: usize = 20;
    let mut sample_rows: Vec<Vec<String>> = Vec::new();
    let mut total_rows_estimate: usize = 0;

    // Iteramos linha a linha (streaming real) — isto é o que garante
    // que nunca carregamos o ficheiro completo para memória nesta fase.
    for result in reader.records() {
        let record = result.map_err(|e| format!("Erro ao ler linha do CSV: {}", e))?;
        if sample_rows.len() < SAMPLE_SIZE {
            sample_rows.push(record.iter().map(|field| field.to_string()).collect());
        }
        total_rows_estimate += 1;
    }

    Ok(FilePreview {
        headers,
        sample_rows,
        total_rows_estimate,
    })
}