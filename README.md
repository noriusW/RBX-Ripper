# RBX Ripper Pro 🚀

[Русский](#russian) | [English](#english)

<a name="russian"></a>
## Русский

**RBX Ripper Pro** — это сверхбыстрый и удобный инструмент для извлечения ресурсов из файлов Roblox (`.rbxlx`). Написан на Rust для обеспечения максимальной производительности и безопасности.

### ✨ Особенности

- ⚡ **Невероятная скорость**: Благодаря Rust и библиотеке `roxmltree`, обработка огромных файлов происходит за считанные секунды.
- 📂 **Полная иерархия**: Программа воссоздает структуру проекта Roblox в виде папок.
- 📜 **Экспорт скриптов**: Все `Script`, `LocalScript` и `ModuleScript` извлекаются в `.lua` файлы.
- ⚙️ **Сохранение свойств**: Все параметры объектов сохраняются в `properties.json`.
- 🌍 **Многоязычность**: Автоматическое определение RU/EN и возможность ручного переключения.
- 🖱️ **Drag-and-Drop**: Просто перетащите файл в окно программы.
- 📊 **Прогресс-бар**: Наглядное отображение процесса извлечения в реальном времени.

### 🚀 Начало работы

#### Загрузка
Вы можете скачать готовую сборку в разделе [Releases](https://github.com/KintaraVault/RBX-Ripper/releases/tag/release).

#### Сборка из исходников
Если у вас установлен Rust, вы можете собрать проект самостоятельно:

```bash
git clone https://github.com/noriusW/RBX-Ripper.git
cd rbx_ripper
cargo run --release
```

---

<a name="english"></a>
## English

**RBX Ripper Pro** is an ultra-fast and user-friendly tool for extracting resources from Roblox place files (`.rbxlx`). Built with Rust to ensure maximum performance and safety.

### ✨ Features

- ⚡ **Blazing Speed**: Powered by Rust and `roxmltree`, it handles massive files in seconds.
- 📂 **Full Hierarchy**: Recreates the Roblox project structure using native folders.
- 📜 **Script Export**: All `Script`, `LocalScript`, and `ModuleScript` objects are extracted as `.lua` files.
- ⚙️ **Property Preservation**: All object properties are saved into `properties.json`.
- 🌍 **Multilingual**: Automatic RU/EN detection with manual toggle support.
- 🖱️ **Drag-and-Drop**: Simply drop your file into the application window.
- 📊 **Progress Visuals**: Real-time progress bar with object counters.

### 🚀 Getting Started

#### Download
Pre-built binaries are available in the [Releases](https://github.com/noriusW/RBX-Ripper/releases/tag/release) section.

#### Building from Source
If you have Rust installed, you can build the project manually:

```bash
git clone https://github.com/noriusW/RBX_Ripper.git
cd rbx_ripper
cargo run --release
```

### 📖 How to Use

1. Launch the application.
2. Drag your `.rbxlx` file into the "Drop here" area or use the "📂 Select File" button.
3. Choose your destination folder (defaults to a new folder next to the source).
4. Click **"Start Extraction"**.
5. Wait for the green checkmark and enjoy!

## 🛠 Tech Stack

- **Language**: Rust
- **GUI**: [egui](https://github.com/emilk/egui)
- **XML Parser**: [roxmltree](https://github.com/RazrFalcon/roxmltree)
- **File Dialogs**: [rfd](https://github.com/Polyfrost/rfd)

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.
