#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use eframe::egui;
use rfd::FileDialog;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use uuid::Uuid;

#[derive(Default)]
struct FFmpegApp {
    video_path: Option<PathBuf>,
    audio_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    delete_orig: bool,
    status_message: String,
}

impl FFmpegApp {
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
        self.video_path = None;
        self.audio_path = None;
        self.output_path = None;
        self.delete_orig = false;
    }

    fn get_default_output_path(&self) -> PathBuf {
        let uuid = Uuid::new_v4().to_string();
        if let Some(video_path) = &self.video_path {
            // 使用视频文件所在的目录
            if let Some(parent) = video_path.parent() {
                return parent.join(format!("{}.mp4", uuid));
            }
        }
        // 如果无法获取视频文件目录，使用当前目录
        PathBuf::from(format!("{}.mp4", uuid))
    }

    fn execute_ffmpeg(&mut self) {
        if let (Some(video), Some(audio)) = (&self.video_path, &self.audio_path) {
            // 如果没有设置输出路径，使用默认路径
            let output = self
                .output_path
                .clone()
                .unwrap_or_else(|| self.get_default_output_path());

            let mut cmd = Command::new("ffmpeg");
            cmd.arg("-i")
                .arg(video)
                .arg("-i")
                .arg(audio)
                .arg("-c")
                .arg("copy")
                .arg(&output);
            #[cfg(windows)]
            cmd.creation_flags(134_217_728u32);

            let result = cmd.output();

            match result {
                Ok(s) => {
                    if s.status.success() {
                        self.status_message = format!("转换成功！输出文件：{}", output.display());

                        if self.delete_orig {
                            // 删除源文件
                            let _ = std::fs::remove_file(self.video_path.as_ref().unwrap());
                            let _ = std::fs::remove_file(self.audio_path.as_ref().unwrap());
                        }
                    } else {
                        self.status_message =
                            format!("转换失败: {}", String::from_utf8_lossy(&s.stderr));
                    }
                }
                Err(e) => {
                    self.status_message = format!("执行错误: {}", e);
                }
            }
        }
    }
}

impl eframe::App for FFmpegApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("FFmpeg 视频/音频合并");
            ui.add_space(10.0);
            ui.label("拖动文件到窗口，自动识别");
            ui.add_space(20.0);

            // 文件拖放处理
            if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
                let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
                for file in dropped_files {
                    if let Some(path) = file.path {
                        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

                        match extension.to_lowercase().as_str() {
                            "mp4" | "mkv" | "avi" if self.video_path.is_none() => {
                                self.video_path = Some(path);
                            }
                            "m4a" | "mp3" | "aac" if self.audio_path.is_none() => {
                                self.audio_path = Some(path);
                            }
                            _ => {}
                        }
                    }
                }
            }

            // 视频文件选择
            ui.horizontal(|ui| {
                if ui.button("选择视频文件").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("视频文件", &["mp4", "mkv", "avi"])
                        .pick_file()
                    {
                        self.video_path = Some(path);
                    }
                }
                if let Some(path) = &self.video_path {
                    ui.label(path.file_name().unwrap().to_string_lossy().to_string());
                }
            });

            ui.add_space(10.0);
            // 音频文件选择
            ui.horizontal(|ui| {
                if ui.button("选择音频文件").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("音频文件", &["m4a", "mp3", "aac"])
                        .pick_file()
                    {
                        self.audio_path = Some(path);
                    }
                }
                if let Some(path) = &self.audio_path {
                    ui.label(path.file_name().unwrap().to_string_lossy().to_string());
                }
            });

            ui.add_space(10.0);
            // 输出文件选择
            ui.horizontal(|ui| {
                if ui.button("选择输出位置").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("MP4文件", &["mp4"])
                        .save_file()
                    {
                        self.output_path = Some(path);
                    }
                }
                if let Some(path) = &self.output_path {
                    ui.label(path.file_name().unwrap().to_string_lossy().to_string());
                }
            });

            ui.add_space(10.0);
            ui.checkbox(&mut self.delete_orig, "完成后删除源文件❗");
            ui.add_space(10.0);

            ui.separator();
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                // 执行按钮
                let can_execute = self.video_path.is_some() && self.audio_path.is_some();

                if ui
                    .add_enabled(can_execute, egui::Button::new("开始处理"))
                    .clicked()
                {
                    self.execute_ffmpeg();
                    self.clear_state();
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
            .with_title("FFmpeg 合并器"),
        ..Default::default()
    };

    eframe::run_native(
        "FFmpeg 合并器",
        native_options,
        Box::new(|cc| Ok(Box::new(FFmpegApp::new(cc)))),
    )
}
