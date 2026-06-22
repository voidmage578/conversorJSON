# 📄 ConversorJSON

Uma aplicação desktop leve para converter ficheiros **Excel (.xlsx)** e **CSV** em ficheiros **JSON** limpos e estruturados.

> ⬇️ [Download da última versão (v0.9)](https://github.com/voidmage578/conversorJSON/releases/tag/v0.9)

---

## ✨ Funcionalidades

- **Carregamento de ficheiros** — drag-and-drop ou diálogo nativo
- **Inferência automática de tipos** — texto, número, booleano e data
- **Mapeamento de colunas** — configura o nome, tipo e estratégia para células vazias de cada coluna
- **Estratégias para células vazias** — Omitir, Nulo ou Valor por defeito
- **Validação** — modo Strict (para ao primeiro erro) ou Tolerant (recolhe todos os erros)
- **Preview de dados** — tabela de pré-visualização antes de exportar
- **Exportação JSON** — ficheiro pretty-printed guardado via diálogo nativo

---

## 🚀 Pipeline

```
Carregamento → Mapeamento & Regras → Validação → Exportação
```

| Fase | Descrição |
|---|---|
| **Carregamento** | Ingestão do ficheiro Excel ou CSV |
| **Mapeamento & Regras** | Configuração por coluna (tipo, nome, células vazias) |
| **Validação** | Verificação de dados em modo Strict ou Tolerant |
| **Exportação** | Geração do JSON final |

---

## 🛠️ Stack Técnica

### Framework
| Tecnologia | Função |
|---|---|
| [Tauri v2](https://tauri.app/) | Shell nativa (Rust + WebView); diálogos e acesso ao sistema de ficheiros |
| [React](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/) | Interface e gestão de estado |
| [Tailwind CSS](https://tailwindcss.com/) | Estilização |

### Frontend
| Biblioteca | Função |
|---|---|
| [SheetJS (`xlsx`)](https://sheetjs.com/) | Parsing de ficheiros Excel no browser |
| [PapaParse](https://www.papaparse.com/) | Parsing de ficheiros CSV |

### Backend (Rust)
| Crate | Função |
|---|---|
| [`calamine`](https://crates.io/crates/calamine) | Leitura de Excel (headers + sample rows, memory-safe) |
| [`csv`](https://crates.io/crates/csv) | Leitura de CSV em streaming |
| [`serde` / `serde_json`](https://serde.rs/) | Serialização entre Rust e TypeScript |
| [`chrono`](https://crates.io/crates/chrono) | Parsing de datas (europeu, ISO 8601, serial Excel) |

---

## 🏗️ Arquitetura

O Rust trata de todo o processamento pesado; o React gere a configuração e a visualização.

```
┌─────────────────────────────────────┐
│           React / TypeScript        │
│  Configuração · Preview · Erros UI  │
└──────────────┬──────────────────────┘
               │ Tauri Commands
┌──────────────▼──────────────────────┐
│              Rust / Tauri           │
│  Leitura · Validação · Exportação   │
└─────────────────────────────────────┘
```

---

## 📦 Instalação

Descarrega o instalador para o teu sistema operativo na página de [Releases](https://github.com/voidmage578/conversorJSON/releases/tag/v0.9).

> **Nota:** Em Windows com Smart App Control ativo, poderá ser necessário confirmar a execução do instalador manualmente.

---

## 🧑‍💻 Desenvolvimento local

### Pré-requisitos

- [Node.js](https://nodejs.org/) + npm
- [Rust](https://www.rust-lang.org/tools/install) + Build Tools (Windows: Visual Studio C++ Build Tools)

### Instalar e correr

```bash
# Instalar dependências frontend
npm install

# Correr em modo desenvolvimento
npm run tauri dev

# Compilar para produção
npm run tauri build
```

---

## 📋 Formatos suportados

| Formato | Extensão |
|---|---|
| Excel | `.xlsx` |
| CSV | `.csv` |

---

## 📄 Licença

Este projeto está disponível sob a licença [MIT](LICENSE).
