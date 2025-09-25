#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::egui::{self, IconData, Pos2, ProgressBar, Ui};
use md5::{Digest, Md5};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use walkdir::WalkDir;

struct RenamerApp {
    paths: Vec<PathBuf>,
    algo: Algo,
    status: String,
    recursive: bool,
    is_processing: bool,
    progress: f32,
    total_files: usize,
    processed_files: usize,
    processing_complete_receiver: Receiver<ProcessingMessage>,
    processing_complete_sender: Sender<ProcessingMessage>,
    processing_thread: Option<JoinHandle<()>>,
    cancel_flag: Arc<Mutex<bool>>,
}

#[derive(Debug)]
enum ProcessingMessage {
    Progress {
        processed: usize,
        total: usize,
    },
    FileProcessed {
        file_name: String,
        success: bool,
        error: Option<String>,
    },
    Complete {
        successful: usize,
        failed: usize,
    },
    Error(String),
}

#[derive(Default, PartialEq, Clone, Copy)]
enum Algo {
    #[default]
    MD5,
    BLAKE3,
}

impl eframe::App for RenamerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(message) = self.processing_complete_receiver.try_recv() {
            self.handle_processing_message(message);
        }

        if self.is_processing {
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hash Renamer");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!self.is_processing, |ui: &mut Ui| ui.button("Select Files"))
                    .clicked()
                    && let Some(files) = rfd::FileDialog::new().pick_files()
                {
                    self.paths = files;
                    self.status = format!("Selected {} file(s)", self.paths.len());
                }

                if ui
                    .add_enabled(!self.is_processing, |ui: &mut Ui| {
                        ui.button("Select Folder")
                    })
                    .clicked()
                    && let Some(folders) = rfd::FileDialog::new().pick_folders()
                {
                    self.paths = folders;
                    self.status = format!("Selected {} folder(s)", self.paths.len());
                }

                if ui
                    .add_enabled(!self.is_processing, |ui: &mut Ui| {
                        ui.button("Clear Selections")
                    })
                    .clicked()
                {
                    self.clear_state();
                }
            });

            ui.add_space(10.0);

            ui.add_enabled_ui(!self.is_processing, |ui: &mut Ui| {
                ui.checkbox(&mut self.recursive, "Recursive folder search");
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Select hash method: ");
                ui.add_enabled_ui(!self.is_processing, |ui| {
                    ui.radio_value(&mut self.algo, Algo::MD5, "MD5");
                    ui.radio_value(&mut self.algo, Algo::BLAKE3, "BLAKE3");
                });
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            if !self.is_processing {
                ui.label("Drag and drop files or folders here");

                let dropped_files = ui.input(|i| i.raw.dropped_files.clone());
                if !dropped_files.is_empty() {
                    self.paths = dropped_files.into_iter().filter_map(|f| f.path).collect();
                    self.status = format!("Dropped {} items", self.paths.len());
                }
            }

            ui.add_space(10.0);

            if self.is_processing {
                ui.vertical(|ui| {
                    ui.label(format!(
                        "Processing: {}/{}",
                        self.processed_files, self.total_files
                    ));

                    let progress_bar = ProgressBar::new(self.progress)
                        .show_percentage()
                        .animate(true);
                    ui.add(progress_bar);
                });
                ui.add_space(10.0);
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                if self.is_processing {
                    if ui.button("Cancel").clicked() {
                        self.cancel_processing();
                    }
                } else if ui
                    .add_enabled(!self.paths.is_empty(), |ui: &mut Ui| {
                        ui.button("Rename Files")
                    })
                    .clicked()
                {
                    self.start_processing();
                }

                ui.add_space(5.0);
                ui.label(&self.status);
            });
        });
    }
}

impl RenamerApp {
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

        let (tx, rx) = channel();
        Self {
            paths: Vec::default(),
            algo: Algo::default(),
            status: String::default(),
            recursive: false,
            is_processing: false,
            progress: 0.0,
            total_files: 0,
            processed_files: 0,
            processing_complete_receiver: rx,
            processing_complete_sender: tx,
            processing_thread: None,
            cancel_flag: Arc::new(Mutex::new(false)),
        }
    }

    fn handle_processing_message(&mut self, message: ProcessingMessage) {
        match message {
            ProcessingMessage::Progress { processed, total } => {
                self.processed_files = processed;
                self.total_files = total;
                self.progress = if total > 0 {
                    processed as f32 / total as f32
                } else {
                    0.0
                };
                self.status = format!("Processing... {processed}/{total}");
            }
            ProcessingMessage::FileProcessed {
                file_name,
                success,
                error,
            } => {
                if !success && let Some(err) = error {
                    eprintln!("Failed to process {file_name}: {err}");
                }
            }
            ProcessingMessage::Complete { successful, failed } => {
                self.is_processing = false;
                self.progress = 1.0;
                self.processing_thread = None;

                if failed > 0 {
                    self.status = format!("Completed: {successful} successful, {failed} failed");
                } else {
                    self.status = format!("Successfully renamed {successful} files");
                }

                self.paths.clear();
            }
            ProcessingMessage::Error(error) => {
                self.is_processing = false;
                self.status = format!("Error: {error}");
                self.processing_thread = None;
            }
        }
    }

    fn start_processing(&mut self) {
        if self.is_processing || self.paths.is_empty() {
            return;
        }

        self.is_processing = true;
        self.progress = 0.0;
        self.processed_files = 0;
        *self.cancel_flag.lock().unwrap() = false;

        let paths = self.paths.clone();
        let algo = self.algo;
        let recursive = self.recursive;
        let sender = self.processing_complete_sender.clone();
        let cancel_flag = Arc::clone(&self.cancel_flag);

        self.processing_thread = Some(thread::spawn(move || {
            Self::process_files_background(paths, algo, recursive, sender, cancel_flag);
        }));

        self.status = "Starting processing...".to_string();
    }

    fn cancel_processing(&mut self) {
        if let Ok(mut flag) = self.cancel_flag.lock() {
            *flag = true;
        }
        self.status = "Cancelling...".to_string();
    }

    fn process_files_background(
        paths: Vec<PathBuf>,
        algo: Algo,
        recursive: bool,
        sender: Sender<ProcessingMessage>,
        cancel_flag: Arc<Mutex<bool>>,
    ) {
        let mut files_to_process = Vec::new();

        for path in &paths {
            if let Ok(flag) = cancel_flag.lock()
                && *flag
            {
                return;
            }

            if path.is_file() {
                files_to_process.push(path.clone());
            } else if path.is_dir() {
                let walker = if recursive {
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

        let total_files = files_to_process.len();
        if total_files == 0 {
            let _ = sender.send(ProcessingMessage::Complete {
                successful: 0,
                failed: 0,
            });
            return;
        }

        let _ = sender.send(ProcessingMessage::Progress {
            processed: 0,
            total: total_files,
        });

        let mut successful = 0;
        let mut failed = 0;

        for (index, file_path) in files_to_process.iter().enumerate() {
            if let Ok(flag) = cancel_flag.lock()
                && *flag
            {
                let _ = sender.send(ProcessingMessage::Error(
                    "Processing cancelled by user".to_string(),
                ));
                return;
            }

            let file_name = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let result = match algo {
                Algo::MD5 => Self::process_file_with_md5(file_path),
                Algo::BLAKE3 => Self::process_file_with_blake3(file_path),
            };

            match result {
                Ok(()) => {
                    successful += 1;
                    let _ = sender.send(ProcessingMessage::FileProcessed {
                        file_name,
                        success: true,
                        error: None,
                    });
                }
                Err(error) => {
                    failed += 1;
                    let _ = sender.send(ProcessingMessage::FileProcessed {
                        file_name,
                        success: false,
                        error: Some(error),
                    });
                }
            }

            let _ = sender.send(ProcessingMessage::Progress {
                processed: index + 1,
                total: total_files,
            });

            thread::sleep(std::time::Duration::from_millis(1));
        }

        let _ = sender.send(ProcessingMessage::Complete { successful, failed });
    }

    fn process_file_with_blake3(file_path: &Path) -> Result<(), String> {
        let file = fs::File::open(file_path).map_err(|e| format!("Failed to open file: {e}"))?;

        let mut reader = std::io::BufReader::with_capacity(1_048_576, file);
        let mut hasher = blake3::Hasher::new();
        let mut buffer = vec![0; 1_048_576];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    hasher.update(&buffer[..n]);
                }
                Err(e) => return Err(format!("Error reading file: {e}")),
            }
        }

        let hash = hasher.finalize();
        let hash_hex = hash.to_hex().to_uppercase();

        Self::rename_file_with_hash(file_path, &hash_hex)
    }

    fn process_file_with_md5(file_path: &Path) -> Result<(), String> {
        let file = fs::File::open(file_path).map_err(|e| format!("Failed to open file: {e}"))?;

        let mut reader = std::io::BufReader::with_capacity(1_048_576, file);
        let mut hasher = Md5::new();
        let mut buffer = vec![0; 1_048_576];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    hasher.update(&buffer[..n]);
                }
                Err(e) => return Err(format!("Error reading file: {e}")),
            }
        }

        let hash = hasher.finalize();
        let hash_hex = format!("{hash:X}");

        Self::rename_file_with_hash(file_path, &hash_hex)
    }

    fn rename_file_with_hash(file_path: &Path, hash_hex: &str) -> Result<(), String> {
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let new_name = if ext.is_empty() {
            hash_hex.to_string()
        } else {
            format!("{hash_hex}.{ext}")
        };

        let new_path = file_path.with_file_name(new_name);

        if new_path.exists() {
            return Err("Target file already exists with the same hash".to_string());
        }

        fs::rename(file_path, new_path).map_err(|e| format!("Failed to rename file: {e}"))
    }

    fn clear_state(&mut self) {
        if self.is_processing {
            return;
        }

        self.paths = Vec::new();
        self.status = String::new();
        self.progress = 0.0;
        self.processed_files = 0;
        self.total_files = 0;
    }
}

impl Drop for RenamerApp {
    fn drop(&mut self) {
        if let Ok(mut flag) = self.cancel_flag.lock() {
            *flag = true;
        }

        if let Some(handle) = self.processing_thread.take() {
            let _ = handle.join();
        }
    }
}

fn main() -> eframe::Result<()> {
    let icon = IconData {
        rgba: get_icon_data().to_vec(),
        width: 256,
        height: 256,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 400.0])
            .with_drag_and_drop(true)
            .with_position(Pos2::new(1000., 600.))
            .with_icon(icon),
        // .with_always_on_top(),
        ..Default::default()
    };

    eframe::run_native(
        "Hash Renamer",
        options,
        Box::new(|cc| Ok(Box::new(RenamerApp::new(cc)))),
    )
}

fn get_icon_data() -> &'static [u8] {
    static IMAGE_BYTES: &[u8] = include_bytes!("../../resources/rnmd/rnmd.png");

    let icon_data = Box::new(
        image::load_from_memory(IMAGE_BYTES)
            .unwrap()
            .to_rgba8()
            .into_raw(),
    );

    Box::leak(icon_data)
}
