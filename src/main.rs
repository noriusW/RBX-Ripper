#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use sys_locale::get_locale;
use rayon::prelude::*;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 500.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    if let Ok(icon_data) = image::open("assets/icon_256x256.png") {
        let icon_data = icon_data.to_rgba8();
        let (width, height) = icon_data.dimensions();
        options.viewport.icon = Some(std::sync::Arc::new(egui::IconData {
            rgba: icon_data.into_raw(),
            width,
            height,
        }));
    }

    eframe::run_native(
        "RBX Ripper Pro",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::new()))),
    )
}

#[derive(Clone, Copy, PartialEq)]
enum Language {
    Russian,
    English,
}

struct Translations {
    title: &'static str,
    drop_here: &'static str,
    select_file: &'static str,
    output_folder: &'static str,
    start: &'static str,
    processing: &'static str,
    done: &'static str,
    error: &'static str,
    language: &'static str,
    reset: &'static str,
    settings: &'static str,
    exclude_workspace: &'static str,
    exclude_scripts: &'static str,
    exclude_classes: &'static str,
}

const RU: Translations = Translations {
    title: "RBX Ripper Pro",
    drop_here: "Перетащите .rbxlx сюда",
    select_file: "Выбрать файл",
    output_folder: "Папка назначения",
    start: "Начать извлечение",
    processing: "Обработка...",
    done: "Готово!",
    error: "Ошибка",
    language: "Язык",
    reset: "Назад",
    settings: "Фильтры извлечения",
    exclude_workspace: "Исключить Workspace",
    exclude_scripts: "Исключить скрипты",
    exclude_classes: "Исключить классы (через запятую):",
};

const EN: Translations = Translations {
    title: "RBX Ripper Pro",
    drop_here: "Drop .rbxlx here",
    select_file: "Select File",
    output_folder: "Output Folder",
    start: "Start Extraction",
    processing: "Processing...",
    done: "Finished!",
    error: "Error",
    language: "Language",
    reset: "Back",
    settings: "Extraction Filters",
    exclude_workspace: "Exclude Workspace",
    exclude_scripts: "Exclude Scripts",
    exclude_classes: "Exclude Classes (comma separated):",
};

#[derive(Clone, PartialEq)]
enum Status {
    Idle,
    Processing { progress: f32, message: String },
    Done(String),
    Error(String),
}

enum LogMessage {
    Progress(f32, String),
    Error(String),
    Finished(String),
}

#[derive(Clone)]
struct ExtractionSettings {
    exclude_workspace: bool,
    exclude_scripts: bool,
    exclude_classes: Vec<String>,
}

struct MyApp {
    status: Status,
    lang: Language,
    last_applied_lang: Option<Language>,
    input_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    rx: Option<Receiver<LogMessage>>,
    exclude_workspace: bool,
    exclude_scripts: bool,
    exclude_classes_input: String,
}

impl MyApp {
    fn new() -> Self {
        let locale = get_locale().unwrap_or_else(|| "en".to_string());
        let lang = if locale.starts_with("ru") {
            Language::Russian
        } else {
            Language::English
        };

        Self {
            status: Status::Idle,
            lang,
            last_applied_lang: None,
            input_path: None,
            output_path: None,
            rx: None,
            exclude_workspace: false,
            exclude_scripts: false,
            exclude_classes_input: String::new(),
        }
    }

    fn t(&self) -> &Translations {
        match self.lang {
            Language::Russian => &RU,
            Language::English => &EN,
        }
    }

    fn start_processing(&mut self, ctx: egui::Context) {
        let input = match &self.input_path {
            Some(p) => p.clone(),
            None => return,
        };
        let output = match &self.output_path {
            Some(p) => p.clone(),
            None => return,
        };

        let (tx, rx) = channel();
        self.rx = Some(rx);
        self.status = Status::Processing { progress: 0.0, message: self.t().processing.to_string() };

        let worker_ctx = ctx.clone();
        let settings = ExtractionSettings {
            exclude_workspace: self.exclude_workspace,
            exclude_scripts: self.exclude_scripts,
            exclude_classes: self.exclude_classes_input.split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect(),
        };

        thread::spawn(move || {
            match process_file_with_progress(&input, &output, &tx, worker_ctx, settings) {
                Ok(count) => {
                    let _ = tx.send(LogMessage::Finished(format!("{} objects", count)));
                }
                Err(e) => {
                    let _ = tx.send(LogMessage::Error(e.to_string()));
                }
            }
        });
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.last_applied_lang != Some(self.lang) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.t().title.to_string()));
            self.last_applied_lang = Some(self.lang);
        }

        let mut finished = false;
        let mut received_msg = false;
        if let Some(rx) = &self.rx {
            while let Ok(msg) = rx.try_recv() {
                received_msg = true;
                match msg {
                    LogMessage::Progress(p, m) => {
                        self.status = Status::Processing { progress: p, message: m };
                    }
                    LogMessage::Error(text) => {
                        self.status = Status::Error(text);
                        finished = true;
                    }
                    LogMessage::Finished(text) => {
                        self.status = Status::Done(text);
                        finished = true;
                    }
                }
            }
        }
        
        if received_msg {
            ctx.request_repaint(); 
        }
        
        if finished { self.rx = None; }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("lang_cb")
                        .selected_text(match self.lang {
                            Language::Russian => "RU",
                            Language::English => "EN",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.lang, Language::Russian, "Русский");
                            ui.selectable_value(&mut self.lang, Language::English, "English");
                        });
                    ui.label(self.t().language);
                });
            });
            ui.add_space(5.0);
        });

        let mut next_status = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            match &self.status {
                Status::Idle => {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.group(|ui| {
                            ui.set_min_height(100.0);
                            ui.vertical_centered(|ui| {
                                ui.add_space(10.0);
                                if ui.add(egui::Button::new(format!("📂 {}", self.t().select_file)).min_size([150.0, 30.0].into())).clicked() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Roblox Place", &["rbxlx"])
                                        .pick_file() {
                                        self.input_path = Some(path.clone());
                                        if self.output_path.is_none() {
                                            let mut out = path.clone();
                                            out.set_extension("");
                                            self.output_path = Some(PathBuf::from(format!("{}_extracted", out.display())));
                                        }
                                    }
                                }
                                ui.add_space(10.0);
                                if let Some(p) = &self.input_path {
                                    ui.strong(format!("📄 {}", p.file_name().unwrap().to_string_lossy()));
                                } else {
                                    ui.label(egui::RichText::new(self.t().drop_here).weak());
                                }
                            });
                        });

                        ui.add_space(10.0);
                        ui.label(format!("{}:", self.t().output_folder));
                        ui.horizontal(|ui| {
                            let path_str = self.output_path.as_ref().map_or("...".to_string(), |p| p.to_string_lossy().to_string());
                            ui.add(egui::Label::new(egui::RichText::new(path_str).monospace()).truncate());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("📂").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                        self.output_path = Some(path);
                                    }
                                }
                            });
                        });

                        ui.separator();
                        let settings_label = self.t().settings;
                        let ex_ws_label = self.t().exclude_workspace;
                        let ex_sc_label = self.t().exclude_scripts;
                        let ex_cl_label = self.t().exclude_classes;

                        ui.collapsing(settings_label, |ui| {
                            ui.checkbox(&mut self.exclude_workspace, ex_ws_label);
                            ui.checkbox(&mut self.exclude_scripts, ex_sc_label);
                            ui.add_space(5.0);
                            ui.label(ex_cl_label);
                            ui.add(egui::TextEdit::singleline(&mut self.exclude_classes_input)
                                .hint_text("Part, MeshPart, Decal...")
                                .desired_width(f32::INFINITY));
                        });

                        ui.add_space(20.0);
                        ui.vertical_centered(|ui| {
                            let can_start = self.input_path.is_some() && self.output_path.is_some();
                            let btn = egui::Button::new(egui::RichText::new(self.t().start).size(18.0).strong())
                                .min_size([200.0, 50.0].into())
                                .fill(egui::Color32::from_rgb(40, 120, 200));
                            
                            if ui.add_enabled(can_start, btn).clicked() {
                                self.start_processing(ctx.clone());
                            }
                        });
                    });
                }
                Status::Processing { progress, message } => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(80.0);
                        ui.heading(self.t().processing);
                        ui.add_space(10.0);
                        ui.add_sized([ui.available_width(), 25.0], egui::ProgressBar::new(*progress)
                            .show_percentage()
                            .animate(false)
                            .rounding(5.0));
                        ui.add_space(10.0);
                        ui.label(message);
                    });
                }
                Status::Done(msg) => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(60.0);
                        let (rect, _) = ui.allocate_at_least(egui::vec2(80.0, 80.0), egui::Sense::hover());
                        let center = rect.center();
                        let radius = 30.0;
                        ui.painter().circle_stroke(center, radius, egui::Stroke::new(3.0, egui::Color32::GREEN));
                        ui.painter().line_segment([
                            center + egui::vec2(-15.0, 0.0),
                            center + egui::vec2(-5.0, 10.0),
                        ], egui::Stroke::new(4.0, egui::Color32::GREEN));
                        ui.painter().line_segment([
                            center + egui::vec2(-5.0, 10.0),
                            center + egui::vec2(15.0, -15.0),
                        ], egui::Stroke::new(4.0, egui::Color32::GREEN));

                        ui.add_space(10.0);
                        ui.heading(self.t().done);
                        ui.label(msg);
                        ui.add_space(20.0);
                        if ui.button(egui::RichText::new(self.t().reset).size(16.0)).clicked() {
                            next_status = Some(Status::Idle);
                        }
                    });
                }
                Status::Error(msg) => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(60.0);
                        let (rect, _) = ui.allocate_at_least(egui::vec2(80.0, 80.0), egui::Sense::hover());
                        let center = rect.center();
                        let radius = 30.0;
                        ui.painter().circle_stroke(center, radius, egui::Stroke::new(3.0, egui::Color32::RED));
                        ui.painter().line_segment([
                            center + egui::vec2(-12.0, -12.0),
                            center + egui::vec2(12.0, 12.0),
                        ], egui::Stroke::new(4.0, egui::Color32::RED));
                        ui.painter().line_segment([
                            center + egui::vec2(12.0, -12.0),
                            center + egui::vec2(-12.0, 12.0),
                        ], egui::Stroke::new(4.0, egui::Color32::RED));

                        ui.add_space(10.0);
                        ui.heading(self.t().error);
                        ui.label(msg);
                        ui.add_space(20.0);
                        if ui.button(egui::RichText::new(self.t().reset).size(16.0)).clicked() {
                            next_status = Some(Status::Idle);
                        }
                    });
                }
            }


            if ctx.input(|i| !i.raw.dropped_files.is_empty()) && self.status == Status::Idle {
                let dropped = ctx.input(|i| i.raw.dropped_files.clone());
                if let Some(file) = dropped.first() {
                    if let Some(path) = &file.path {
                        if path.extension().map_or(false, |e| e == "rbxlx") {
                            self.input_path = Some(path.clone());
                            let mut out = path.clone();
                            out.set_extension("");
                            self.output_path = Some(PathBuf::from(format!("{}_extracted", out.display())));
                        }
                    }
                }
            }
        });

        if let Some(s) = next_status {
            self.status = s;
        }
    }
}

fn process_file_with_progress(input: &Path, output: &Path, tx: &Sender<LogMessage>, ctx: egui::Context, settings: ExtractionSettings) -> anyhow::Result<usize> {
    let text = fs::read_to_string(input)?;
    let doc = roxmltree::Document::parse(&text)?;
    
    let top_items: Vec<_> = doc.root().children().filter(|n| n.has_tag_name("Item")).collect();
    let roblox_node = doc.root().children().find(|n| n.has_tag_name("roblox"));
    let mut all_top_items = top_items;
    if let Some(r) = roblox_node {
        all_top_items.extend(r.children().filter(|n| n.has_tag_name("Item")));
    }

    let mut total_items = 0;
    for node in &all_top_items {
        total_items += count_items_recursive(*node, &settings);
    }

    if total_items == 0 {
        return Ok(0);
    }

    fs::create_dir_all(output)?;
    let current_count = Arc::new(AtomicUsize::new(0));
    
    all_top_items.into_par_iter().try_for_each(|node| {
        process_item_recursive_parallel(node, output, &current_count, total_items, tx, &ctx, &settings)
    })?;
    
    ctx.request_repaint();
    Ok(total_items)
}

fn count_items_recursive(node: roxmltree::Node, settings: &ExtractionSettings) -> usize {
    if should_exclude_node(node, settings) {
        return 0;
    }
    
    let mut count = 1;
    for child in node.children().filter(|n| n.has_tag_name("Item")) {
        count += count_items_recursive(child, settings);
    }
    count
}

fn should_exclude_node(node: roxmltree::Node, settings: &ExtractionSettings) -> bool {
    let class_name = node.attribute("class").unwrap_or("Unknown");
    
    if settings.exclude_classes.contains(&class_name.to_lowercase()) {
        return true;
    }

    if settings.exclude_scripts {
        if class_name == "Script" || class_name == "LocalScript" || class_name == "ModuleScript" {
            return true;
        }
    }

    if settings.exclude_workspace {
        if let Some(props_node) = node.children().find(|n| n.has_tag_name("Properties")) {
            for prop in props_node.children() {
                if prop.attribute("name") == Some("Name") {
                    if prop.text() == Some("Workspace") {
                        return true;
                    }
                }
            }
        }
    }

    false
}

fn process_item_recursive_parallel(
    node: roxmltree::Node, 
    parent_path: &Path, 
    current: &Arc<AtomicUsize>, 
    total: usize, 
    tx: &Sender<LogMessage>,
    ctx: &egui::Context,
    settings: &ExtractionSettings
) -> anyhow::Result<()> {
    if should_exclude_node(node, settings) {
        return Ok(());
    }

    let count = current.fetch_add(1, Ordering::SeqCst) + 1;
    
    if count % 20 == 0 || count == total {
        let progress = count as f32 / total as f32;
        if tx.send(LogMessage::Progress(progress, format!("{} / {}", count, total))).is_ok() {
            ctx.request_repaint();
        }
    }

    let class_name = node.attribute("class").unwrap_or("Unknown");
    let mut name = class_name.to_string();
    let mut source_code = None;
    let mut properties = serde_json::Map::new();
    
    properties.insert("ClassName".to_string(), serde_json::Value::String(class_name.to_string()));

    if let Some(props_node) = node.children().find(|n| n.has_tag_name("Properties")) {
        for prop in props_node.children() {
            if !prop.is_element() { continue; }
            let prop_name = prop.attribute("name").unwrap_or("Unknown");
            if prop_name == "Source" {
                source_code = prop.text().map(|s| s.to_string());
                continue;
            }
            if prop_name == "Name" {
                if let Some(text) = prop.text() {
                    name = text.to_string();
                }
            }
            if let Some(text) = prop.text() {
                properties.insert(prop_name.to_string(), serde_json::Value::String(text.to_string()));
            }
        }
    }
    
    let safe_name = sanitize_filename::sanitize(&name);
    let folder_name = if safe_name.to_lowercase() == class_name.to_lowercase() {
        safe_name
    } else {
        format!("{} [{}]", safe_name, class_name)
    };
    let mut target_dir = parent_path.join(&folder_name);
    
    let mut i = 1;
    while target_dir.exists() {
        target_dir = parent_path.join(format!("{} ({})", folder_name, i));
        i += 1;
    }
    
    fs::create_dir_all(&target_dir)?;
    
    let props_path = target_dir.join("properties.json");
    let mut writer = BufWriter::new(fs::File::create(props_path)?);
    serde_json::to_writer_pretty(&mut writer, &properties)?;
    writer.flush()?;
    
    if let Some(source) = source_code {
        let mut s_writer = BufWriter::new(fs::File::create(target_dir.join("script.lua"))?);
        s_writer.write_all(source.as_bytes())?;
        s_writer.flush()?;
    }
    
    let children: Vec<_> = node.children().filter(|n| n.has_tag_name("Item")).collect();
    
    children.into_par_iter().try_for_each(|child| {
        process_item_recursive_parallel(child, &target_dir, current, total, tx, ctx, settings)
    })?;
    
    Ok(())
}
