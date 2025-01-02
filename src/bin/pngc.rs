#![cfg(feature = "pngc")]
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::egui;
use rfd::FileDialog;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

#[derive(Default)]
struct PngCompress {
    image_path: Option<Vec<PathBuf>>,
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
        let mut fonts = egui::FontDefinitions::default();

        // 添加系统字体
        fonts.font_data.insert(
            "my_font".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "../../resources/SarasaUiSC-Regular.ttf"
            ))),
        );

        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "my_font".to_owned());

        cc.egui_ctx.set_fonts(fonts);

        Default::default()
    }

    fn clear_state(&mut self) {
        self.image_path = None;
    }

    fn execute_oxipng(&mut self) {
        if let Some(image) = &self.image_path {
            let mut cmd = Command::new("oxipng");
            cmd.arg("-s").arg("-o2").arg("-t4");

            for i in image {
                cmd.arg(i);
            }

            let result = cmd.creation_flags(134_217_728u32).output();

            match result {
                Ok(s) => {
                    if s.status.success() {
                        self.status_message = "优化成功！".to_string();
                        // 处理完成后清除状态
                        self.clear_state();
                    } else {
                        self.status_message =
                            format!("优化失败: {}", String::from_utf8_lossy(&s.stderr));
                    }
                }
                Err(e) => {
                    self.status_message = format!("执行错误: {}", e);
                }
            }
        }
    }
}

impl eframe::App for PngCompress {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Oxipng 图像优化");
            ui.add_space(10.0);
            ui.label("拖动文件到窗口，自动识别");
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
                self.status_message = format!("选择了{}个文件", png_images.len());
                self.image_path = Some(png_images);
            }

            // 选择文件
            ui.horizontal(|ui| {
                if ui.button("选择文件").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("图像文件", &["png", "PNG"])
                        .pick_files()
                    {
                        self.status_message = format!("选择了{}个文件", path.len());
                        self.image_path = Some(path);
                    }
                }
            });

            ui.add_space(10.0);
            ui.label("优化参数: -s -o2 -t4");
            ui.add_space(10.0);

            ui.separator();

            ui.add_space(20.0);
            ui.horizontal(|ui| {
                // 执行按钮
                let can_execute = self.image_path.is_some();

                if ui
                    .add_enabled(can_execute, egui::Button::new("开始处理"))
                    .clicked()
                {
                    self.execute_oxipng();
                }

                // 清除按钮
                if ui.button("清除选择").clicked() {
                    self.clear_state();
                }
            });

            // 状态信息显示
            if !self.status_message.is_empty() {
                ui.label(&self.status_message);
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 320.0])
            .with_title("Oxipng 图像优化"),
        ..Default::default()
    };

    eframe::run_native(
        "Oxipng 图像优化",
        native_options,
        Box::new(|cc| Ok(Box::new(PngCompress::new(cc)))),
    )
}
