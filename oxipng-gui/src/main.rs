#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::egui::{self, IconData, Pos2};
use rayon::{ThreadPoolBuilder, prelude::*};
use rfd::FileDialog;
use std::path::PathBuf;

#[derive(Default)]
struct PngCompress {
    image_path: Option<Vec<PathBuf>>,
    opt_lvl: u8,
    status_message: String,
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

        Self {
            opt_lvl: 2,
            ..Default::default()
        }
    }

    fn clear_state(&mut self) {
        self.image_path = None;
    }

    fn execute_oxipng(&mut self) {
        let pool = ThreadPoolBuilder::new().num_threads(8).build().unwrap();

        pool.install(|| {
            if let Some(image) = &self.image_path {
                image.par_iter().for_each(|path| {
                    let _ = oxipng::optimize(
                        &oxipng::InFile::Path(path.to_path_buf()),
                        &oxipng::OutFile::Path {
                            path: None,
                            preserve_attrs: true,
                        },
                        &oxipng::Options::from_preset(2),
                    );
                });
            }
        });

        self.status_message = "Optimize Success!".to_string();
    }
}

impl eframe::App for PngCompress {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Oxipng Optimizer");
            ui.add_space(20.0);

            // 文件拖放处理
            if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
                let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
                let png_images: Vec<PathBuf> = dropped_files
                    .into_iter()
                    .filter_map(|f| f.path)
                    .filter(|f| {
                        f.extension()
                            .and_then(|ext| ext.to_str())
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
                    })
                    .collect();
                self.status_message = format!("Select {} file(s)", png_images.len());
                self.image_path = Some(png_images);
            }

            // 选择文件
            ui.horizontal(|ui| {
                if ui.button("Select files").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("PNG Images", &["png", "PNG"])
                        .pick_files()
                    {
                        self.status_message = format!("Select {} file(s)", path.len());
                        self.image_path = Some(path);
                    }
                }

                // 清除按钮
                if ui.button("Clear").clicked() {
                    self.clear_state();
                }
            });

            ui.add_space(10.0);

            ui.label(format!("Current: Preset {}", self.opt_lvl));
            ui.add(egui::Slider::new(&mut self.opt_lvl, 0..=6).text("Preset level"));

            ui.add_space(10.0);

            ui.separator();

            ui.add_space(10.0);
            ui.label("Drag and Drop files here");
            ui.add_space(20.0);
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                // 执行按钮
                let can_execute = self.image_path.is_some();

                if ui
                    .add_enabled(can_execute, egui::Button::new("Process"))
                    .clicked()
                {
                    self.execute_oxipng();
                }

                // 状态信息显示
                if !self.status_message.is_empty() {
                    ui.label(&self.status_message);
                }
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
            .with_inner_size([450.0, 320.0])
            .with_title("Oxipng Optimizer")
            .with_position(Pos2::new(1000., 600.))
            .with_always_on_top()
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
