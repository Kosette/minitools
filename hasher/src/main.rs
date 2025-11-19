#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::{
    NativeOptions,
    egui::{self, IconData},
};
use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

const BUFFER_SIZE: usize = 4 * 1024 * 1024;

// I18n support
#[derive(Clone, Copy, PartialEq)]
enum Language {
    English,
    Chinese,
}

#[derive(Clone, Copy)]
struct I18n {
    lang: Language,
}

impl I18n {
    fn new() -> Self {
        // Detect system language
        let lang = sys_locale::get_locale()
            .map(|locale| {
                if locale.starts_with("zh") {
                    Language::Chinese
                } else {
                    Language::English
                }
            })
            .unwrap_or(Language::English);

        Self { lang }
    }

    fn t<'a>(&self, key: &'a str) -> &'a str {
        match (self.lang, key) {
            (Language::English, "title") => "File Hash Calculator",
            (Language::Chinese, "title") => "文件哈希计算器",
            (Language::English, "instruction") => "Drag and Drop file, or click button",
            (Language::Chinese, "instruction") => "拖放文件，或点击按钮",
            (Language::English, "select_file") => "Select file...",
            (Language::Chinese, "select_file") => "选择文件...",
            (Language::English, "save_txt") => "Save to TXT",
            (Language::Chinese, "save_txt") => "保存为TXT",
            (Language::English, "save_csv") => "Save to CSV",
            (Language::Chinese, "save_csv") => "保存为CSV",
            (Language::English, "clear_all") => "Clear All",
            (Language::Chinese, "clear_all") => "清除全部",
            (Language::English, "file") => "File",
            (Language::Chinese, "file") => "文件",
            (Language::English, "copy_clipboard") => "Copy to clipboard",
            (Language::Chinese, "copy_clipboard") => "复制到剪贴板",
            (Language::English, "processing_error") => "Processing Error",
            (Language::Chinese, "processing_error") => "处理错误",
            (Language::English, "language") => "Language",
            (Language::Chinese, "language") => "语言",
            _ => key,
        }
    }
}

struct MyApp {
    results: Vec<FileHashResult>,
    last_error: Option<String>,
    i18n: I18n,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            results: Vec::new(),
            last_error: None,
            i18n: I18n::new(),
        }
    }
}

struct FileHashResult {
    path: PathBuf,
    sha1: String,
    sha256: String,
    sha512: String,
    md5: String,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles = [
            (
                egui::TextStyle::Heading,
                egui::FontId::new(20.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Button,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Body,
                egui::FontId::new(14.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Monospace,
                egui::FontId::new(14.0, egui::FontFamily::Proportional),
            ),
        ]
        .into();
        cc.egui_ctx.set_style(style);

        // 设置默认字体以支持中文
        let mut fonts = egui::FontDefinitions::default();

        // 添加系统字体
        fonts.font_data.insert(
            "chinese_font".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "../../resources/fonts/SimHei.ttf"
            ))),
        );

        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "chinese_font".to_owned());

        cc.egui_ctx.set_fonts(fonts);

        Self {
            ..Default::default()
        }
    }

    fn compute_hashes(path: &PathBuf) -> std::io::Result<FileHashResult> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut sha1 = Sha1::new();
        let mut sha256 = Sha256::new();
        let mut sha512 = Sha512::new();
        let mut md5 = Md5::new();

        let mut buffer = vec![0u8; BUFFER_SIZE];
        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            let chunk = &buffer[..n];
            sha1.update(chunk);
            sha256.update(chunk);
            sha512.update(chunk);
            md5.update(chunk);
        }

        Ok(FileHashResult {
            path: path.clone(),
            sha1: format!("{:x}", sha1.finalize()),
            sha256: format!("{:x}", sha256.finalize()),
            sha512: format!("{:x}", sha512.finalize()),
            md5: format!("{:x}", md5.finalize()),
        })
    }

    fn save_results_txt(&self) {
        use rfd::FileDialog;
        if let Some(path) = FileDialog::new()
            .set_file_name("hash_results.txt")
            .add_filter("文本文件", &["txt"])
            .save_file()
        {
            let mut out = String::new();
            for r in &self.results {
                out.push_str(&format!("File: {}\n", r.path.display()));
                out.push_str(&format!("  SHA1   : {}\n", r.sha1));
                out.push_str(&format!("  SHA256 : {}\n", r.sha256));
                out.push_str(&format!("  SHA512 : {}\n", r.sha512));
                out.push_str(&format!("  MD5    : {}\n\n", r.md5));
            }
            if let Err(e) = std::fs::write(&path, out) {
                eprintln!("Failed to Save to File: {}", e);
            }
        }
    }

    fn csv_escape(s: &str) -> String {
        let mut t = String::from("\"");
        for ch in s.chars() {
            if ch == '"' {
                t.push('"');
            }
            t.push(ch);
        }
        t.push('"');
        t
    }

    fn save_results_csv(&self) {
        use rfd::FileDialog;
        if let Some(path) = FileDialog::new()
            .set_file_name("hash_results.csv")
            .add_filter("CSV 文件", &["csv"])
            .save_file()
        {
            let mut out = String::from("File,SHA1,SHA256,SHA512,MD5\n");
            for r in &self.results {
                out.push_str(&format!(
                    "{},{},{},{},{}\n",
                    Self::csv_escape(&r.path.display().to_string()),
                    Self::csv_escape(&r.sha1),
                    Self::csv_escape(&r.sha256),
                    Self::csv_escape(&r.sha512),
                    Self::csv_escape(&r.md5)
                ));
            }
            if let Err(e) = std::fs::write(&path, out) {
                eprintln!("Failed to Save to CSV: {}", e);
            }
        }
    }

    fn handle_one_file(&mut self, path: PathBuf) {
        match Self::compute_hashes(&path) {
            Ok(res) => {
                self.results.push(res);
                self.last_error = None;
            }
            Err(e) => {
                self.last_error = Some(format!(
                    "{}：{} ({})",
                    self.i18n.t("processing_error"),
                    path.display(),
                    e
                ));
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(self.i18n.t("title"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(self.i18n.t("language"));
                    if ui
                        .selectable_label(self.i18n.lang == Language::English, "English")
                        .clicked()
                    {
                        self.i18n.lang = Language::English;
                    }
                    if ui
                        .selectable_label(self.i18n.lang == Language::Chinese, "中文")
                        .clicked()
                    {
                        self.i18n.lang = Language::Chinese;
                    }
                });
            });
            ui.add_space(8.0);
            ui.label(self.i18n.t("instruction"));

            ui.add_space(8.0);

            if ui.button(self.i18n.t("select_file")).clicked()
                && let Some(path) = rfd::FileDialog::new().pick_file()
            {
                self.handle_one_file(path);
            }

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        !self.results.is_empty(),
                        egui::Button::new(self.i18n.t("save_txt")),
                    )
                    .clicked()
                {
                    self.save_results_txt();
                }
                if ui
                    .add_enabled(
                        !self.results.is_empty(),
                        egui::Button::new(self.i18n.t("save_csv")),
                    )
                    .clicked()
                {
                    self.save_results_csv();
                }
                if ui
                    .add_enabled(
                        !self.results.is_empty(),
                        egui::Button::new(self.i18n.t("clear_all")),
                    )
                    .clicked()
                {
                    self.results.clear();
                    self.last_error = None;
                }
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            let dropped = ctx.input(|i| i.raw.dropped_files.clone());
            if !dropped.is_empty() {
                for f in dropped {
                    if let Some(path) = f.path {
                        self.handle_one_file(path);
                    }
                }
            }

            if let Some(err) = &self.last_error {
                ui.colored_label(egui::Color32::RED, err);
                ui.add_space(6.0);
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                for r in &self.results {
                    ui.group(|ui| {
                        ui.label(format!("{}: {}", self.i18n.t("file"), r.path.display()));
                        ui.monospace(String::from("--------"));

                        ui.horizontal(|ui| {
                            ui.monospace(format!("SHA1   : {}", r.sha1));
                            if ui
                                .button("📋")
                                .on_hover_text(self.i18n.t("copy_clipboard"))
                                .clicked()
                            {
                                ui.ctx().copy_text(r.sha1.clone());
                            }
                        });
                        ui.monospace(String::from("--------"));

                        ui.horizontal(|ui| {
                            ui.monospace(format!("SHA256 : {}", r.sha256));
                            if ui
                                .button("📋")
                                .on_hover_text(self.i18n.t("copy_clipboard"))
                                .clicked()
                            {
                                ui.ctx().copy_text(r.sha256.clone());
                            }
                        });
                        ui.monospace(String::from("--------"));

                        ui.horizontal(|ui| {
                            ui.monospace(format!("SHA512 : {}", r.sha512));
                            if ui
                                .button("📋")
                                .on_hover_text(self.i18n.t("copy_clipboard"))
                                .clicked()
                            {
                                ui.ctx().copy_text(r.sha512.clone());
                            }
                        });
                        ui.monospace(String::from("--------"));

                        ui.horizontal(|ui| {
                            ui.monospace(format!("MD5    : {}", r.md5));
                            if ui
                                .button("📋")
                                .on_hover_text(self.i18n.t("copy_clipboard"))
                                .clicked()
                            {
                                ui.ctx().copy_text(r.md5.clone());
                            }
                        });
                    });
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let icon = IconData {
        rgba: get_icon_data().to_vec(),
        width: 256,
        height: 256,
    };

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1050.0, 600.0])
            .with_title("Hash Calc")
            .with_icon(icon),
        ..Default::default()
    };
    eframe::run_native(
        "Hash Calc",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

fn get_icon_data() -> &'static [u8] {
    use image;
    static IMAGE_BYTES: &[u8] = include_bytes!("../../resources/hasher/hasher.png");
    let icon_data = Box::new(
        image::load_from_memory(IMAGE_BYTES)
            .unwrap()
            .to_rgba8()
            .into_raw(),
    );
    Box::leak(icon_data)
}
