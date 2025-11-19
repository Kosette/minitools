#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::egui::{self, IconData, ScrollArea};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

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
            (Language::English, "title") => "yt-dlp downloader",
            (Language::Chinese, "title") => "yt-dlp 下载器",
            (Language::English, "paste_url") => "Paste URL here / One per line:",
            (Language::Chinese, "paste_url") => "在此粘贴URL / 每行一个：",
            (Language::English, "download") => "Download",
            (Language::Chinese, "download") => "下载",
            (Language::English, "clear") => "Clear",
            (Language::Chinese, "clear") => "清除",
            (Language::English, "cancel") => "Cancel",
            (Language::Chinese, "cancel") => "取消",
            (Language::English, "logs") => "Logs:",
            (Language::Chinese, "logs") => "日志：",
            (Language::English, "no_url") => "No URL found.",
            (Language::Chinese, "no_url") => "未找到URL。",
            (Language::English, "cancelling") => "Cancelling...",
            (Language::Chinese, "cancelling") => "正在取消...",
            (Language::English, "start_downloading") => "Start downloading",
            (Language::Chinese, "start_downloading") => "开始下载",
            (Language::English, "downloading_complete") => "Downloading complete",
            (Language::Chinese, "downloading_complete") => "下载完成",
            (Language::English, "execution_failed") => "Execution failed",
            (Language::Chinese, "execution_failed") => "执行失败",
            (Language::English, "downloading_cancelled") => "Downloading cancelled",
            (Language::Chinese, "downloading_cancelled") => "下载已取消",
            (Language::English, "all_complete") => "=== All Complete! ===",
            (Language::Chinese, "all_complete") => "=== 全部完成！ ===",
            (Language::English, "language") => "Language",
            (Language::Chinese, "language") => "语言",
            _ => key,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let icon = IconData {
        rgba: get_icon_data().to_vec(),
        width: 256,
        height: 256,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([550.0, 500.0])
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "BDown",
        options,
        Box::new(|cc| Ok(Box::new(YTDlpGui::new(cc)))),
    )
}

struct YTDlpGui {
    input_text: String,
    logs: Arc<Mutex<Vec<String>>>,
    is_downloading: bool,
    cancel_flag: Arc<Mutex<bool>>,
    download_done: Arc<Mutex<bool>>,
    i18n: I18n,
}
impl YTDlpGui {
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

        Self::default()
    }

    fn clear_state(&mut self) {
        self.input_text = String::new();
        self.logs = Arc::new(Mutex::new(Vec::new()));
    }
}

impl Default for YTDlpGui {
    fn default() -> Self {
        Self {
            input_text: String::new(),
            logs: Arc::new(Mutex::new(Vec::new())),
            is_downloading: false,
            cancel_flag: Arc::new(Mutex::new(false)),
            download_done: Arc::new(Mutex::new(false)),
            i18n: I18n::new(),
        }
    }
}

impl eframe::App for YTDlpGui {
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
            ui.add_space(10.0);
            ui.label(self.i18n.t("paste_url"));
            ui.add_space(10.0);

            ui.add_sized(
                [ui.available_width(), 150.0],
                egui::TextEdit::multiline(&mut self.input_text),
            );

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if !self.is_downloading {
                    if ui.button(self.i18n.t("download")).clicked() {
                        let urls: Vec<String> = self
                            .input_text
                            .lines()
                            .map(str::trim)
                            .filter(|line| !line.is_empty())
                            .map(String::from)
                            .collect();

                        if !urls.is_empty() {
                            let logs = Arc::clone(&self.logs);
                            let cancel_flag = Arc::clone(&self.cancel_flag);
                            let download_done = Arc::clone(&self.download_done);
                            *cancel_flag.lock().unwrap() = false;
                            *download_done.lock().unwrap() = false;
                            self.is_downloading = true;

                            let ctx_clone = ctx.clone();
                            let i18n = self.i18n;

                            tokio::spawn(async move {
                                run_downloads(urls, logs, cancel_flag, ctx_clone, i18n).await;
                                *download_done.lock().unwrap() = true;
                            });
                        } else {
                            self.logs.lock().unwrap().push(self.i18n.t("no_url").into());
                        }
                    }

                    if ui.button(self.i18n.t("clear")).clicked() {
                        self.clear_state();
                    }
                } else if ui.button(self.i18n.t("cancel")).clicked() {
                    *self.cancel_flag.lock().unwrap() = true;
                    self.logs
                        .lock()
                        .unwrap()
                        .push(self.i18n.t("cancelling").into());
                }
            });

            if self.is_downloading && *self.download_done.lock().unwrap() {
                self.is_downloading = false;
                self.logs
                    .lock()
                    .unwrap()
                    .push(self.i18n.t("all_complete").into());
            }

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(15.0);
            ui.label(self.i18n.t("logs"));

            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    let logs = self.logs.lock().unwrap();
                    for line in logs.iter() {
                        ui.label(line);
                    }
                });
        });

        ctx.request_repaint();
    }
}

async fn run_downloads(
    urls: Vec<String>,
    logs: Arc<Mutex<Vec<String>>>,
    cancel_flag: Arc<Mutex<bool>>,
    ctx: egui::Context,
    i18n: I18n,
) {
    for url in urls {
        {
            if *cancel_flag.lock().unwrap() {
                let mut logs = logs.lock().unwrap();
                logs.push(i18n.t("downloading_cancelled").into());
                break;
            }
        }

        {
            let mut logs = logs.lock().unwrap();
            logs.push(format!("{} {url}", i18n.t("start_downloading")));
        }
        ctx.request_repaint();

        let mut cmd = Command::new("yt-dlp");
        cmd.arg(&url).stdout(Stdio::piped()).stderr(Stdio::piped());

        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let output = cmd.output().await;

        match output {
            Ok(out) => {
                let mut logs = logs.lock().unwrap();
                if !out.stdout.is_empty() {
                    logs.push(String::from_utf8_lossy(&out.stdout).to_string());
                }
                if !out.stderr.is_empty() {
                    logs.push(String::from_utf8_lossy(&out.stderr).to_string());
                }
                logs.push(i18n.t("downloading_complete").into());
            }
            Err(e) => {
                let mut logs = logs.lock().unwrap();
                logs.push(format!("{}: {e}", i18n.t("execution_failed")));
            }
        }

        ctx.request_repaint();
    }
}

fn get_icon_data() -> &'static [u8] {
    static IMAGE_BYTES: &[u8] = include_bytes!("../../resources/bdown/bdown.png");

    let icon_data = Box::new(
        image::load_from_memory(IMAGE_BYTES)
            .unwrap()
            .to_rgba8()
            .into_raw(),
    );

    Box::leak(icon_data)
}
