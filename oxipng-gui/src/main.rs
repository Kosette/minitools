#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::egui::{self, IconData, Pos2, Ui};
use rayon::prelude::*;
use rfd::FileDialog;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
    mpsc::{Receiver, Sender, channel},
};
use std::thread;
use walkdir::WalkDir;

enum Message {
    Update(String),
    Progress(f32),
    Finished,
}

struct PngCompress {
    image_paths: Option<Vec<PathBuf>>,
    opt_lvl: u8,
    status_message: String,
    is_optimizing: bool,
    progress: f32,
    channel_rx: Receiver<Message>,
    channel_tx: Sender<Message>,
    num_threads: usize,
    max_threads: usize,
    recursive_search: bool,
}

impl PngCompress {
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
        let max_threads = num_cpus::get();

        Self {
            opt_lvl: 2,
            image_paths: None,
            status_message: "Select or drop files/folders.".to_string(),
            is_optimizing: false,
            progress: 0.0,
            channel_rx: rx,
            channel_tx: tx,
            num_threads: (max_threads / 2).max(1),
            max_threads,
            recursive_search: false,
        }
    }

    fn clear_state(&mut self) {
        self.image_paths = None;
        self.status_message = "Select or drop files/folders.".to_string();
        self.progress = 0.0;
    }

    fn execute_oxipng(&mut self, ctx: egui::Context) {
        if let Some(paths) = self.image_paths.clone() {
            self.is_optimizing = true;
            self.progress = 0.0;
            let sender = self.channel_tx.clone();
            let opt_lvl = self.opt_lvl;
            let num_threads = self.num_threads;
            let total_files = paths.len();
            let processed_count = Arc::new(AtomicUsize::new(0));

            thread::spawn(move || {
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(num_threads)
                    .build()
                    .unwrap();
                pool.install(|| {
                    paths.par_iter().for_each(|path| {
                        let options = oxipng::Options::from_preset(opt_lvl);
                        let in_file = oxipng::InFile::Path(path.clone());
                        let out_file = oxipng::OutFile::Path {
                            path: None,
                            preserve_attrs: true,
                        };
                        let message = match oxipng::optimize(&in_file, &out_file, &options) {
                            Ok(_) => format!("Optimized: {}", path.display()),
                            Err(e) => format!("Error on {}: {}", path.display(), e),
                        };
                        let current_processed = processed_count.fetch_add(1, Ordering::SeqCst) + 1;
                        sender.send(Message::Update(message)).ok();
                        sender
                            .send(Message::Progress(
                                current_processed as f32 / total_files as f32,
                            ))
                            .ok();
                        ctx.request_repaint();
                    });
                });
                sender.send(Message::Finished).ok();
                ctx.request_repaint();
            });
        }
    }

    fn process_input_paths(&mut self, paths: Vec<PathBuf>) {
        let mut png_files = HashSet::new();
        for path in paths {
            if path.is_dir() {
                let mut walker = WalkDir::new(&path).into_iter();
                if !self.recursive_search {
                    walker = WalkDir::new(path).max_depth(1).into_iter();
                }

                for entry in walker.filter_map(|e| e.ok()) {
                    let entry_path = entry.path();
                    if entry_path.is_file()
                        && entry_path
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
                    {
                        png_files.insert(entry_path.to_path_buf());
                    }
                }
            } else if path.is_file()
                && path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
            {
                png_files.insert(path);
            }
        }

        if !png_files.is_empty() {
            self.status_message =
                format!("Found {} PNG file(s). Ready to process.", png_files.len());
            self.image_paths = Some(png_files.into_iter().collect());
        } else {
            self.status_message = "No PNG files found in the selection.".to_string();
            self.image_paths = None;
        }
    }
}

impl eframe::App for PngCompress {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(msg) = self.channel_rx.try_recv() {
            match msg {
                Message::Update(status) => self.status_message = status,
                Message::Progress(val) => self.progress = val,
                Message::Finished => {
                    self.is_optimizing = false;
                    self.status_message = format!(
                        "Optimization complete for {} files!",
                        self.image_paths.as_ref().map_or(0, |v| v.len())
                    );
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Oxipng Optimizer");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!self.is_optimizing, |ui: &mut Ui| ui.button("Select files"))
                    .clicked()
                {
                    if let Some(paths) = FileDialog::new()
                        .add_filter("PNG Images", &["png", "PNG"])
                        .pick_files()
                    {
                        self.process_input_paths(paths);
                    }
                }
                if ui
                    .add_enabled(!self.is_optimizing, |ui: &mut Ui| {
                        ui.button("Select Folder")
                    })
                    .clicked()
                {
                    if let Some(path) = FileDialog::new().pick_folder() {
                        self.process_input_paths(vec![path]);
                    }
                }
                if ui
                    .add_enabled(!self.is_optimizing, |ui: &mut Ui| ui.button("Clear"))
                    .clicked()
                {
                    self.clear_state();
                }
            });

            if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
                let dropped_paths: Vec<PathBuf> = ctx
                    .input(|i| i.raw.dropped_files.clone())
                    .into_iter()
                    .filter_map(|f| f.path)
                    .collect();
                if !dropped_paths.is_empty() {
                    self.process_input_paths(dropped_paths);
                }
            }

            ui.add_space(10.0);

            ui.label(format!("Current Preset Level: {}", self.opt_lvl));
            ui.add_enabled(
                !self.is_optimizing,
                egui::Slider::new(&mut self.opt_lvl, 0..=6).text("Optimization Level"),
            );
            ui.add_space(5.0);
            ui.label(format!("Threads to use: {}", self.num_threads));
            ui.add_enabled(
                !self.is_optimizing,
                egui::Slider::new(&mut self.num_threads, 1..=self.max_threads).text("Thread Count"),
            );

            ui.add_enabled(
                !self.is_optimizing,
                egui::Checkbox::new(
                    &mut self.recursive_search,
                    "Search in subfolders (Recursive)",
                ),
            );

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.vertical_centered(|ui| {
                ui.label("Drag and Drop files or folders here");
            });
            ui.add_space(20.0);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                if ui
                    .add_enabled(
                        self.image_paths.is_some() && !self.is_optimizing,
                        egui::Button::new("Process"),
                    )
                    .clicked()
                {
                    self.execute_oxipng(ctx.clone());
                }
                ui.add_space(5.0);
                if self.is_optimizing {
                    ui.add(egui::ProgressBar::new(self.progress).show_percentage());
                }
                ui.add_space(5.0);
                ui.label(&self.status_message);
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let icon = IconData {
        rgba: get_icon_data().to_vec(),
        width: 32,
        height: 32,
    };
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([470.0, 420.0])
            .with_title("Oxipng Optimizer")
            .with_position(Pos2::new(1000., 600.))
            // .with_always_on_top()
            .with_icon(icon),
        ..Default::default()
    };
    eframe::run_native(
        "Oxipng Optimizer",
        native_options,
        Box::new(|cc| Ok(Box::new(PngCompress::new(cc)))),
    )
}

fn get_icon_data() -> &'static [u8] {
    use image;
    static IMAGE_BYTES: &[u8] = include_bytes!("../../resources/icon.png");
    let icon_data = Box::new(
        image::load_from_memory(IMAGE_BYTES)
            .unwrap()
            .to_rgba8()
            .into_raw(),
    );
    Box::leak(icon_data)
}
