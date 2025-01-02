#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use eframe::egui;
use rayon::prelude::*;
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
        // 设置文本样式
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
        // let mut fonts = egui::FontDefinitions::default();

        // // 添加系统字体
        // fonts.font_data.insert(
        //     "my_font".to_owned(),
        //     std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
        //         "../resources/SarasaUiSC-Regular.ttf"
        //     ))),
        // );

        // fonts
        //     .families
        //     .get_mut(&egui::FontFamily::Proportional)
        //     .unwrap()
        //     .insert(0, "my_font".to_owned());

        // cc.egui_ctx.set_fonts(fonts);

        Self {
            opt_lvl: 2,
            ..Default::default()
        }
    }

    fn clear_state(&mut self) {
        self.image_path = None;
    }

    fn execute_oxipng(&mut self) {
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
                            .map_or(false, |ext| ext.eq_ignore_ascii_case("png"))
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
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 320.0])
            .with_title("Oxipng Optimizer"),
        ..Default::default()
    };

    eframe::run_native(
        "Oxipng Optimizer",
        native_options,
        Box::new(|cc| Ok(Box::new(PngCompress::new(cc)))),
    )
}
