# Conversor de Ficheiros XLS/XLSX/CSV para JSON

Aplicação desktop construída com Tauri v2 que permite converter ficheiros Excel ou CSV num array JSON validado.  
Inclui dois modos de validação (Tolerante e Estrito), preview dos dados, regras para células vazias e exportação final do ficheiro JSON.

---

## ✨ Funcionalidades Principais

- **Upload de ficheiros** (`.xls`, `.xlsx`, `.csv`)
- **Preview dos dados** antes da validação
- **Configuração de regras para células vazias**
- **Dois modos de validação:**
  - **Modo Tolerante**  
    - Analisa o ficheiro completo  
    - Gera relatório com:  
      - número total de linhas  
      - erros encontrados  
      - linhas onde ocorreram  
    - Permite exportar JSON omitindo as linhas com erro
  - **Modo Estrito**  
    - Interrompe a análise ao primeiro erro  
    - Permite recarregar o ficheiro corrigido
- **Exportação final para JSON** (pretty‑printed)
- **Interface moderna** com React + Tailwind
- **Diálogos nativos** para abrir/guardar ficheiros

---

## 🧱 Arquitetura & Stack

A aplicação segue uma separação clara entre **frontend (React/TS)** e **backend nativo (Rust via Tauri)**.

### 🖥️ Frontend (React + TypeScript)

- **React + TypeScript** — UI e gestão de estado
- **Tailwind CSS** — estilização
- **SheetJS (xlsx)** — parsing de ficheiros Excel no browser
- **PapaParse** — parsing de CSV no browser

Responsabilidades:
- Configuração de mapeamento
- Preview dos dados
- Regras para células vazias
- UI de erros e validação
- Interação com o backend via Tauri

---

### ⚙️ Backend (Rust via Tauri v2)

- **Tauri v2** — shell nativa, diálogos de ficheiros, acesso ao sistema
- **calamine** — leitura de Excel (headers + sample rows), seguro para ficheiros grandes
- **csv crate** — leitura de CSV em streaming
- **serde / serde_json** — serialização/deserialização
- **chrono** — parsing de datas (europeu, ISO 8601, serial Excel)

Responsabilidades:
- Leitura de ficheiros (Excel/CSV)
- Validação (Strict/Tolerant)
- Normalização de dados
- Exportação do JSON final

---

## 🔄 Pipeline da Aplicação

1. **Carregamento do ficheiro**  
   - Drag‑and‑drop ou diálogo nativo

2. **Mapeamento & Regras**  
   - Inferência de tipos  
   - Estratégias para células vazias  
   - Preview dos dados

3. **Validação**  
   - **Strict Mode:** pára no primeiro erro  
   - **Tolerant Mode:** percorre tudo e gera relatório

4. **Exportação**  
   - JSON formatado  
   - Diálogo nativo para guardar o ficheiro

---

## 🚀 Como Executar

1. Fazer download da Release mais recente  
2. Executar o ficheiro `.exe`  
3. Carregar um ficheiro Excel/CSV  
4. Seguir os passos de validação e exportar o JSON

> Não é necessário instalar nada — a aplicação é standalone.

---

## 📦 Download

👉 **[Download da versão mais recente]([#](https://github.com/voidmage578/conversorJSON/releases/tag/v0.9))**

---

## 📌 Possíveis Melhorias Futuras

- Suporte a múltiplos schemas de validação
- Exportação para outros formatos (YAML, XML)
- Editor visual de regras
- Histórico de ficheiros convertidos
- Modo CLI (linha de comandos)

---

## 📜 Licença

Este projeto é distribuído para fins educativos e demonstração técnica.

