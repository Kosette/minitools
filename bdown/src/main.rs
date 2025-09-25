#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::egui::{self, IconData, ScrollArea};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

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

        // // 设置默认字体以支持中文
        // let mut fonts = egui::FontDefinitions::default();
        //
        // // 添加系统字体
        // fonts.font_data.insert(
        //     "my_font".to_owned(),
        //     std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
        //         "../../resources/SimHei.ttf"
        //     ))),
        // );
        //
        // fonts
        //     .families
        //     .get_mut(&egui::FontFamily::Proportional)
        //     .unwrap()
        //     .insert(0, "my_font".to_owned());
        //
        // cc.egui_ctx.set_fonts(fonts);
        //
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
        }
    }
}

impl eframe::App for YTDlpGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("yt-dlp downloader");
            ui.add_space(10.0);
            ui.label("Paste URL here / One per line: ");
            ui.add_space(10.0);

            ui.add_sized(
                [ui.available_width(), 150.0],
                egui::TextEdit::multiline(&mut self.input_text),
            );

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if !self.is_downloading {
                    if ui.button("Download").clicked() {
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

                            tokio::spawn(async move {
                                run_downloads(urls, logs, cancel_flag, ctx_clone).await;
                                *download_done.lock().unwrap() = true;
                            });
                        } else {
                            self.logs.lock().unwrap().push("No URL found.".into());
                        }
                    }

                    if ui.button("Clear").clicked() {
                        self.clear_state();
                    }
                } else if ui.button("Cancel").clicked() {
                    *self.cancel_flag.lock().unwrap() = true;
                    self.logs.lock().unwrap().push("Cancelling...".into());
                }
            });

            if self.is_downloading && *self.download_done.lock().unwrap() {
                self.is_downloading = false;
                self.logs
                    .lock()
                    .unwrap()
                    .push("=== All Complete! ===".into());
            }

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(15.0);
            ui.label("Logs: ");

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
) {
    for url in urls {
        {
            if *cancel_flag.lock().unwrap() {
                let mut logs = logs.lock().unwrap();
                logs.push("Downloading cancelled".into());
                break;
            }
        }

        {
            let mut logs = logs.lock().unwrap();
            logs.push(format!("Start downloading {url}"));
        }
        ctx.request_repaint();

        let output = Command::new("yt-dlp")
            .arg(&url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .await;

        match output {
            Ok(out) => {
                let mut logs = logs.lock().unwrap();
                if !out.stdout.is_empty() {
                    logs.push(String::from_utf8_lossy(&out.stdout).to_string());
                }
                if !out.stderr.is_empty() {
                    logs.push(String::from_utf8_lossy(&out.stderr).to_string());
                }
                logs.push("Downloading complete".into());
            }
            Err(e) => {
                let mut logs = logs.lock().unwrap();
                logs.push(format!("Excution failed: {e}"));
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
