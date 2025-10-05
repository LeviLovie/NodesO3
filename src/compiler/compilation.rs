use chrono::{DateTime, Utc};
use eframe::egui::{Context, Window};
use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, process::Command, rc::Rc, time::Duration};

use crate::file_picker::{DialogPurpose, FilePicker};

pub const AYU_DARK: ColorTheme = ColorTheme {
    name: "Ayu Dark",
    dark: true,
    bg: "#0f1419",
    cursor: "#bfbdb6",
    selection: "#ffad66",
    comments: "#5c6773",
    functions: "#e6b450",
    keywords: "#ffad66",
    literals: "#bd552f",
    numerics: "#dfbfff",
    punctuation: "#bfbdb6",
    strs: "#aad94c",
    types: "#59c2ff",
    special: "#f28779",
};

impl Default for Compilation {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Compilation {
    pub code: String,
    pub timestamp: DateTime<Utc>,
    pub elapsed_times: Vec<(String, Duration)>,
    #[serde(skip)]
    save_picker: Rc<RefCell<Option<FilePicker>>>,
    run_result: Option<String>,
}

impl Compilation {
    pub fn new() -> Self {
        Self {
            code: String::new(),
            timestamp: Utc::now(),
            elapsed_times: Vec::new(),
            save_picker: Rc::new(RefCell::new(None)),
            run_result: None,
        }
    }

    pub fn add_elapsed_time(&mut self, stage: &str, duration: Duration) {
        self.elapsed_times.push((stage.to_string(), duration));
    }

    pub fn set_code(&mut self, code: String) {
        self.code = code;
    }

    pub fn update(&mut self, ctx: &Context) {
        let mut reset_picker = false;
        if let Some(picker) = self.save_picker.borrow_mut().as_mut() {
            picker.show(ctx);
            if let Some(path) = &picker.picked_path {
                if let Err(e) = std::fs::write(path, &self.code) {
                    eprintln!("Failed to save compilation code: {}", e);
                } else {
                    reset_picker = true;
                }
            }
        }
        if reset_picker {
            *self.save_picker.borrow_mut() = None;
        }

        Window::new("Compilation").resizable(true).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("{}", self.timestamp));
                for (stage, duration) in &self.elapsed_times {
                    ui.label(stage.to_string());
                    ui.label(format!("{:?}", duration));
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    *self.save_picker.borrow_mut() =
                        Some(FilePicker::new(DialogPurpose::SavePython));
                }

                if ui.button("Run").clicked() {
                    let output = Command::new("python").arg("-c").arg(&self.code).output();
                    self.run_result = match output {
                        Ok(output) => {
                            let mut combined = Vec::new();
                            combined.extend_from_slice(&output.stdout);
                            combined.extend_from_slice(&output.stderr);
                            Some(String::from_utf8_lossy(&combined).to_string())
                        }
                        Err(e) => Some(format!("Failed to run code: {}", e)),
                    };
                }

                #[cfg(feature = "rustpython")]
                {
                    if ui.button("RustPython Run").clicked() {
                        use rustpython::{
                            vm::{PyObjectRef, VirtualMachine},
                            InterpreterConfig,
                        };
                        use std::sync::{Arc, Mutex};

                        let output = Arc::new(Mutex::new(String::new()));

                        let interp = InterpreterConfig::new().init_stdlib().interpreter();
                        let _ = interp.enter(|vm| {
                            let scope = vm.new_scope_with_builtins();

                            let output_clone = output.clone();
                            let print_obj = vm.new_function(
                                "print",
                                move |args: Vec<PyObjectRef>, vm: &VirtualMachine| {
                                    let string = args
                                        .into_iter()
                                        .map(|obj| {
                                            let py_str = obj.str(vm).unwrap();
                                            py_str.as_str().to_string()
                                        })
                                        .collect::<Vec<_>>()
                                        .join("");
                                    output_clone.lock().unwrap().push_str(&string);
                                    output_clone.lock().unwrap().push('\n');
                                    vm.ctx.none()
                                },
                            );
                            vm.builtins.set_attr("print", print_obj, vm).unwrap();

                            match vm.run_code_string(scope, &self.code, "main".to_string()) {
                                Ok(_) => "Code ran successfully".to_string(),
                                Err(py_err) => {
                                    vm.print_exception(py_err);
                                    String::new()
                                }
                            }
                        });

                        self.run_result = Some(output.lock().unwrap().clone());
                    }
                }
            });

            ui.add_space(6.0);

            CodeEditor::default()
                .id_source("code editor")
                .with_rows(16)
                .with_fontsize(12.0)
                .with_theme(ColorTheme::GRUVBOX)
                .with_syntax(Syntax::python())
                .with_numlines(true)
                .show(ui, &mut self.code);

            if self.run_result.is_some() {
                ui.separator();
                if let Some(result) = &mut self.run_result {
                    let mut result = result.clone();
                    CodeEditor::default()
                        .id_source("result editor")
                        .with_rows(6)
                        .with_fontsize(12.0)
                        .with_theme(AYU_DARK)
                        .with_numlines(false)
                        .show(ui, &mut result);
                }
            }
        });
    }
}
