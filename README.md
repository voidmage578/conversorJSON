# 📄 ConversorJSON

A lightweight desktop application for converting **Excel (.xlsx)** and **CSV** files into clean, validated and structured **JSON**.  
Built with **Tauri v2**, combining Rust performance with a modern React-based interface.

> ⬇️ [Download latest version (v0.9)](https://github.com/voidmage578/conversorJSON/releases/tag/v0.9)

---

## ✨ Features

- **File loading** — drag‑and‑drop or native file dialog
- **Automatic type inference** — text, number, boolean, date
- **Column mapping** — rename fields, set types and empty‑cell strategies
- **Empty‑cell strategies** — Omit · Null · Default value
- **Data validation**
  - **Strict Mode** — stops at the first error and allows re‑uploading a corrected file
  - **Tolerant Mode** — scans the entire file and generates a full error report
- **Data preview** — interactive table before exporting
- **JSON export** — pretty‑printed output saved via native dialog

---

## 🚀 Pipeline

```
Load → Mapping & Rules → Validation → Export
```

| Stage | Description |
|-------|-------------|
| **Load** | Reads the Excel or CSV file |
| **Mapping & Rules** | Per‑column configuration (type, name, empty‑cell strategy) |
| **Validation** | Strict (interrupts) or Tolerant (full report) |
| **Export** | Generates the final JSON |

---

## 🛠️ Tech Stack

### Framework
| Technology | Purpose |
|------------|----------|
| **Tauri v2** | Native shell (Rust + WebView); dialogs and filesystem access |
| **React + TypeScript** | UI and state management |
| **Tailwind CSS** | Styling |

### Frontend
| Library | Purpose |
|---------|----------|
| **SheetJS (`xlsx`)** | Excel parsing in the browser |
| **PapaParse** | CSV parsing |

### Backend (Rust)
| Crate | Purpose |
|--------|---------|
| **calamine** | Excel reading (headers + sample rows) |
| **csv** | Streaming CSV reader |
| **serde / serde_json** | Serialization between Rust and TypeScript |
| **chrono** | Date parsing (EU, ISO 8601, Excel serial) |

---

## 🏗️ Architecture

Rust handles all heavy processing; React manages configuration and UI.

```
┌─────────────────────────────────────┐
│           React / TypeScript        │
│   Mapping · Preview · Error UI      │
└──────────────┬──────────────────────┘
               │ Tauri Commands
┌──────────────▼──────────────────────┐
│              Rust / Tauri           │
│  Reading · Validation · Export      │
└─────────────────────────────────────┘
```

---

## 📦 Installation

Download the installer for your OS from the [Releases](https://github.com/voidmage578/conversorJSON/releases/tag/v0.9) page.

> ⚠️ **Note:** On Windows with Smart App Control enabled, you may need to manually confirm the installer.

---

## 🧑‍💻 Local Development

### Requirements
- Node.js + npm  
- Rust + Build Tools (Windows: Visual Studio C++ Build Tools)

### Install & run

```bash
npm install
npm run tauri dev
npm run tauri build
```

---

## 📋 Supported Formats

| Format | Extension |
|---|---|
| Excel | `.xlsx` |
| CSV | `.csv` |

---

## 📄 License

This project is available under the [MIT](LICENSE).
