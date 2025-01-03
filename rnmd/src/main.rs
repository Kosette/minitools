#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use eframe::egui;
use md5::{Digest, Md5};
use rayon::prelude::*;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Default)]
struct RenamerApp {
    paths: Vec<PathBuf>,
    algo: Algo,
    status: String,
    recursive: bool,
}

#[derive(Default, PartialEq)]
enum Algo {
    MD5,
    #[default]
    BLAKE3,
}

impl eframe::App for RenamerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hash Renamer");

            ui.add_space(10.0);

            // File selection buttons
            ui.horizontal(|ui| {
                if ui.button("Select Files").clicked() {
                    if let Some(files) = rfd::FileDialog::new().pick_files() {
                        self.paths = files;
                        self.status = format!("Selected {} file(s)", self.paths.len());
                    }
                }
                if ui.button("Select Folder").clicked() {
                    if let Some(folders) = rfd::FileDialog::new().pick_folders() {
                        self.paths = folders;
                        self.status = format!("Selected {} folder(s)", self.paths.len());
                    }
                }

                if ui.button("Clear Selections").clicked() {
                    self.clear_state();
                }
            });

            ui.add_space(10.0);
            // Recursive option
            ui.checkbox(&mut self.recursive, "Recursive folder search");

            ui.add_space(10.0);
            // hash algorithm
            ui.horizontal(|ui| {
                ui.label("Select hash method: ");
                ui.radio_value(&mut self.algo, Algo::MD5, "md5");
                ui.radio_value(&mut self.algo, Algo::BLAKE3, "blake3");
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.label("Drag and drop files or folders here");

            let dropped_files = ui.input(|i| i.raw.dropped_files.clone());
            if !dropped_files.is_empty() {
                self.paths = dropped_files.into_iter().filter_map(|f| f.path).collect();
                self.status = format!("Dropped {} items", self.paths.len());
            }

            ui.add_space(10.0);
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                if ui
                    .add_enabled(!self.paths.is_empty(), egui::Button::new("Rename Files"))
                    .clicked()
                {
                    self.rename_files();
                }
                ui.label(&self.status);
            });
        });
    }
}

impl RenamerApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Set style
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

        Default::default()
    }

    fn rename_files(&mut self) {
        let mut files_to_process = Vec::new();

        // Collect all files
        for path in &self.paths {
            if path.is_file() {
                files_to_process.push(path.clone());
            } else if path.is_dir() {
                let walker = if self.recursive {
                    WalkDir::new(path)
                } else {
                    WalkDir::new(path).max_depth(1)
                };

                files_to_process.extend(
                    walker
                        .into_iter()
                        .filter_map(Result::ok)
                        .filter(|e| e.file_type().is_file())
                        .map(|e| e.path().to_path_buf()),
                );
            }
        }

        // Process files in parallel
        match &self.algo {
            Algo::MD5 => {
                let results: Vec<_> = files_to_process
                    .par_iter()
                    .filter_map(|file| self.process_file_with_md5(file))
                    .collect();

                self.status = format!("Renamed {} files", results.len());
            }
            Algo::BLAKE3 => {
                let results: Vec<_> = files_to_process
                    .par_iter()
                    .filter_map(|file| self.process_file_with_blake3(file))
                    .collect();

                self.status = format!("Renamed {} files", results.len());
            }
        }

        self.paths.clear();
    }

    fn process_file_with_blake3(&self, file_path: &Path) -> Option<()> {
        let file = fs::File::open(file_path).ok()?;
        let mut reader = std::io::BufReader::with_capacity(5_242_880, file);

        let mut hasher = blake3::Hasher::new();

        let mut buffer = vec![0; 5_242_880];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    hasher.update(&buffer[..n]);
                }
                Err(e) => panic!("Error reading file: {}", e),
            }
        }

        let hash = hasher.finalize();

        let hash_hex = hash.to_hex().to_uppercase();

        // Create new filename
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let new_name = if ext.is_empty() {
            hash_hex
        } else {
            format!("{}.{}", hash_hex, ext)
        };

        let new_path = file_path.with_file_name(new_name);

        // Skip if target file already exists
        if new_path.exists() {
            return None;
        }

        // Rename file
        fs::rename(file_path, new_path).ok()
    }

    fn process_file_with_md5(&self, file_path: &Path) -> Option<()> {
        let file = fs::File::open(file_path).ok()?;
        let mut reader = std::io::BufReader::with_capacity(5_242_880, file);

        let mut hasher = Md5::new();

        let mut buffer = vec![0; 5_242_880];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    hasher.update(&buffer[..n]);
                }
                Err(e) => panic!("Error reading file: {}", e),
            }
        }

        let hash = hasher.finalize();

        let hash_hex = format!("{:X}", hash);

        // Create new filename
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let new_name = if ext.is_empty() {
            hash_hex
        } else {
            format!("{}.{}", hash_hex, ext)
        };

        let new_path = file_path.with_file_name(new_name);

        // Skip if target file already exists
        if new_path.exists() {
            return None;
        }

        // Rename file
        fs::rename(file_path, new_path).ok()
    }

    fn clear_state(&mut self) {
        self.paths = Vec::new();
        self.status = String::new();
        self.recursive = false;
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([440.0, 330.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "Hash Renamer",
        options,
        Box::new(|cc| Ok(Box::new(RenamerApp::new(cc)))),
    )
}
