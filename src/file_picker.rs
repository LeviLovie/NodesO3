use eframe::egui::Context;
use egui_file_dialog::{FileDialog, FileDialogConfig};
use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogPurpose {
    OpenWorkspace,
    SaveWorkspace,
    ImportLibs,
    SavePython,
}

#[derive(Debug)]
pub struct FilePicker {
    dlg: Option<FileDialog>,
    activated: bool,
    pub purpose: DialogPurpose,
    pub picked_path: Option<PathBuf>,
    pub picked_paths: Option<Vec<PathBuf>>,
}

impl FilePicker {
    pub fn new(purpose: DialogPurpose) -> Self {
        let mut dlg = FileDialog::new();

        let mut dlg_config = FileDialogConfig::default();
        match purpose {
            DialogPurpose::OpenWorkspace => {
                dlg_config.title = Some("Open a Workspace".to_string());
                dlg_config.default_file_filter = Some("NodesO₃ Workspaces".to_string());
            }
            DialogPurpose::SaveWorkspace => {
                dlg_config = dlg_config.add_save_extension("NodesO₃ Workspace", "no3");
                dlg_config =
                    dlg_config.add_save_extension("NodesO₃ Compressed Workspace", "no3zstd");
                dlg_config.default_save_extension = Some("no3".to_string());
                dlg_config.title = Some("Save to a Workspace".to_string());
            }
            DialogPurpose::ImportLibs => {
                dlg_config.title = Some("Import a Library".to_string());
                dlg_config.default_file_filter = Some("NodesO₃ Libraries".to_string());
            }
            DialogPurpose::SavePython => {
                dlg_config = dlg_config.add_save_extension("Python", "py");
                dlg_config.default_save_extension = Some("py".to_string());
                dlg_config.title = Some("Save to a Python File".to_string());
            }
        }
        *dlg.config_mut() = dlg_config;

        match purpose {
            DialogPurpose::OpenWorkspace => {
                dlg = dlg.add_file_filter_extensions("NodesO₃ Workspaces", vec!["no3", "no3zstd"]);
            }
            DialogPurpose::ImportLibs => {
                dlg = dlg.add_file_filter(
                    "NodesO₃ Libraries",
                    Arc::new(|p| format!("{p:?}").ends_with(".no3lib.yaml\"")),
                );
            }
            _ => {}
        }

        Self {
            dlg: Some(dlg),
            activated: false,
            purpose,
            picked_path: None,
            picked_paths: None,
        }
    }

    pub fn show(&mut self, ctx: &Context) {
        let mut picked_paths: Option<Vec<PathBuf>> = None;

        if let Some(ref mut dlg) = self.dlg {
            if !self.activated {
                self.activated = true;

                match self.purpose {
                    DialogPurpose::OpenWorkspace => dlg.pick_file(),
                    DialogPurpose::SaveWorkspace => dlg.save_file(),
                    DialogPurpose::ImportLibs => dlg.pick_multiple(),
                    DialogPurpose::SavePython => dlg.save_file(),
                };
            }

            dlg.update(ctx);

            match self.purpose {
                DialogPurpose::ImportLibs => {
                    if let Some(path_bufs) = dlg.picked_multiple() {
                        picked_paths = Some(path_bufs.iter().map(|p| p.to_path_buf()).collect());
                    }
                }
                _ => {
                    if let Some(path_buf) = dlg.picked() {
                        picked_paths = Some(vec![path_buf.to_path_buf()]);
                    }
                }
            }
        }

        if let Some(picked_paths) = picked_paths {
            self.picked_paths = Some(picked_paths.clone());
            self.picked_path = Some(picked_paths.first().unwrap().to_path_buf());
            self.dlg = None;
        }
    }
}
