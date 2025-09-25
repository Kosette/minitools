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

#[derive(Default)]
struct MyApp {
    results: Vec<FileHashResult>,
    last_error: Option<String>,
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
                self.last_error = Some(format!("Processing Error：{} ({})", path.display(), e));
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("File Hash Calculator");
            ui.add_space(8.0);
            ui.label("Drag and Drop file, or click button");

            ui.add_space(8.0);

            if ui.button("Select file...").clicked()
                && let Some(path) = rfd::FileDialog::new().pick_file()
            {
                self.handle_one_file(path);
            }

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if ui.button("Save to TXT").clicked() {
                    self.save_results_txt();
                }
                if ui.button("Save to CSV").clicked() {
                    self.save_results_csv();
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
                        ui.label(format!("File: {}", r.path.display()));
                        ui.monospace(String::from("--------"));
                        ui.monospace(format!("SHA1   : {}", r.sha1));
                        ui.monospace(String::from("--------"));
                        ui.monospace(format!("SHA256 : {}", r.sha256));
                        ui.monospace(String::from("--------"));
                        ui.monospace(format!("SHA512 : {}", r.sha512));
                        ui.monospace(String::from("--------"));
                        ui.monospace(format!("MD5    : {}", r.md5));
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
            .with_inner_size([600.0, 500.0])
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
