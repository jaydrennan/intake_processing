// src/app.rs

use crate::models::{CheckItem, Document};
use crate::pdf_handler::PDFHandler;
use eframe::egui;
use std::collections::HashMap;
use chrono::Utc;
use egui::TextureHandle;
use indexmap::IndexMap;
use rfd::FileDialog;

pub struct ChecklistApp {
    documents: HashMap<String, Document>,
    current_document: String,
    pdf_handler: Option<PDFHandler>,
    selected_section: Option<usize>,
    new_doc_name: String,
    rename_state: Option<(String, String)>,
}

impl ChecklistApp {
    pub fn new() -> Self {
        let mut app = Self {
            documents: HashMap::new(),
            current_document: String::new(),
            pdf_handler: None,
            selected_section: None,
            new_doc_name: String::new(),
            rename_state: None,
        };
        
        // Load saved checklists
        if let Ok(saved) = app.load_state() {
            app.documents = saved;
            if let Some(first) = app.documents.keys().next() {
                app.current_document = first.clone();
            }
        }
        
        app
    }

    fn save_state(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.documents)
            .map_err(|e| e.to_string())?;
        std::fs::write("checklists.json", json)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn load_state(&self) -> Result<HashMap<String, Document>, String> {
        let data = std::fs::read_to_string("checklists.json")
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&data)
            .map_err(|e| e.to_string())
    }

    fn handle_pdf_upload(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("PDF", &["pdf"])
            .pick_file()
        {
            if let Ok(mut handler) = PDFHandler::new() {
                if handler.load_pdf(path).is_ok() {
                    self.pdf_handler = Some(handler);
                }
            }
        }
    }
}

impl eframe::App for ChecklistApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut save_needed = false;

        // Left panel for document list
        egui::SidePanel::left("documents_panel").show(ctx, |ui| {
            ui.heading("Checklists");
            
            // New checklist input and button
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.new_doc_name);
                if ui.button("New Checklist").clicked() && !self.new_doc_name.is_empty() {
                    let name = self.new_doc_name.clone();
                    let doc = Document {
                        name: name.clone(),
                        checklist: create_default_checklist(),
                        last_modified: Utc::now(),
                    };
                    self.documents.insert(name.clone(), doc);
                    self.current_document = name;
                    self.new_doc_name.clear();
                    save_needed = true;
                }
            });

            ui.separator();

            // List of existing checklists with rename buttons
            let mut to_rename = None;
            for name in self.documents.keys() {
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.current_document == *name, name).clicked() {
                        self.current_document = name.clone();
                    }
                    if ui.small_button("✏").clicked() {
                        to_rename = Some((name.clone(), name.clone()));
                    }
                });
            }

            // Handle rename if requested
            if let Some((old_name, new_name)) = to_rename {
                self.rename_state = Some((old_name, new_name));
            }

            ui.separator();
            if ui.button("Upload PDF").clicked() {
                self.handle_pdf_upload();
            }
        });

        // Rename popup
        if let Some((old_name, mut new_name)) = self.rename_state.clone() {
            egui::Window::new("Rename Checklist")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut new_name);
                        if ui.button("Save").clicked() && !new_name.is_empty() {
                            if let Some(doc) = self.documents.remove(&old_name) {
                                let mut updated_doc = doc;
                                updated_doc.name = new_name.clone();
                                self.documents.insert(new_name.clone(), updated_doc);
                                if self.current_document == old_name {
                                    self.current_document = new_name;
                                }
                                save_needed = true;
                            }
                            self.rename_state = None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.rename_state = None;
                        }
                    });
                });
        }

        // Main panel with checklist
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(doc) = self.documents.get_mut(&self.current_document) {
                ui.columns(2, |columns| {
                    // Left column: Checklist
                    columns[0].heading("Checklist");
                    for (section, item) in doc.checklist.iter_mut() {
                        columns[0].collapsing(section, |ui| {
                            for (child_label, child) in item.children.iter_mut() {
                                if ui.checkbox(&mut child.checked, child_label).clicked() {
                                    if let Some(handler) = &self.pdf_handler {
                                        for (idx, pdf_section) in handler.get_sections().iter().enumerate() {
                                            if pdf_section.title == *section {
                                                self.selected_section = Some(idx);
                                            }
                                        }
                                    }
                                }
                            }
                        });
                    }

                    // Right column: PDF Preview
                    columns[1].heading("PDF Section");
                    if let Some(handler) = &self.pdf_handler {
                        if let Some(idx) = self.selected_section {
                            if let Some(section) = handler.get_sections().get(idx) {
                                columns[1].label(&section.title);
                                if let Some(image) = &section.image {
                                    let texture: TextureHandle = ctx.load_texture(
                                        format!("section_{}", idx),
                                        egui::ColorImage::from_rgba_unmultiplied(
                                            [image.width() as _, image.height() as _],
                                            image.as_raw(),
                                        ),
                                        Default::default(),
                                    );
                                    
                                    egui::ScrollArea::both()
                                        .show(&mut columns[1], |ui| {
                                            ui.add(egui::Image::new(&texture)
                                                .fit_to_original_size(1.0)
                                                .maintain_aspect_ratio(true));
                                        });
                                } else {
                                    columns[1].label("No image available for this section");
                                }
                            }
                        } else {
                            columns[1].label("Select a checklist item to view the corresponding PDF section");
                        }
                    } else {
                        columns[1].label("No PDF loaded. Upload a PDF using the button in the left panel.");
                    }
                });
            }
        });

        if save_needed {
            let _ = self.save_state();
        }
    }
}

fn create_default_checklist() -> IndexMap<String, CheckItem> {
    let mut checklist = IndexMap::new();
    
    // Home Tab
    let mut home = CheckItem::new();
    home.children.insert("Update Status to Treating".to_string(), CheckItem::new());
    home.children.insert("Primary Contact: Attorney Assigned".to_string(), CheckItem::new());
    home.children.insert("Case Manager: Secretary for attorney assigned".to_string(), CheckItem::new());
    home.children.insert("Lead Attorney: Attorney assigned".to_string(), CheckItem::new());
    home.children.insert("Supervising Attorney: Should be blank".to_string(), CheckItem::new());
    checklist.insert("Home Tab".to_string(), home);
    
    // Client Tab
    let mut client = CheckItem::new();
    
    // Index Information
    let mut index_info = CheckItem::new();
    index_info.children.insert("DOB".to_string(), CheckItem::new());
    index_info.children.insert("SSN".to_string(), CheckItem::new());
    index_info.children.insert("Date of Incident".to_string(), CheckItem::new());
    index_info.children.insert("Check name spelling".to_string(), CheckItem::new());
    index_info.children.insert("Check DOL".to_string(), CheckItem::new());
    client.children.insert("Index Information".to_string(), index_info);
    
    // Contact Details
    let mut contact = CheckItem::new();
    contact.children.insert("Cell".to_string(), CheckItem::new());
    contact.children.insert("Email".to_string(), CheckItem::new());
    contact.children.insert("Home Address".to_string(), CheckItem::new());
    
    // Spouse Information
    let mut spouse = CheckItem::new();
    spouse.children.insert("Marital Status".to_string(), CheckItem::new());
    spouse.children.insert("First Name".to_string(), CheckItem::new());
    spouse.children.insert("Last Name".to_string(), CheckItem::new());
    spouse.children.insert("Phone Number".to_string(), CheckItem::new());
    spouse.children.insert("OK to Discuss".to_string(), CheckItem::new());
    contact.children.insert("Spouse".to_string(), spouse);
    
    // Emergency Contact
    let mut emergency = CheckItem::new();
    emergency.children.insert("First Name".to_string(), CheckItem::new());
    emergency.children.insert("Last Name".to_string(), CheckItem::new());
    emergency.children.insert("Phone Number".to_string(), CheckItem::new());
    contact.children.insert("Emergency Contact".to_string(), emergency);
    
    client.children.insert("Contact Details".to_string(), contact);
    checklist.insert("Client Tab".to_string(), client);
    
    // Defendant Tab
    let mut defendant = CheckItem::new();
    defendant.children.insert("Defendant Name".to_string(), CheckItem::new());
    defendant.children.insert("Insurance Company".to_string(), CheckItem::new());
    defendant.children.insert("Policy Number".to_string(), CheckItem::new());
    defendant.children.insert("Claim Number".to_string(), CheckItem::new());
    checklist.insert("Defendant Tab".to_string(), defendant);
    
    // Incident Tab
    let mut incident = CheckItem::new();
    incident.children.insert("Date of Incident".to_string(), CheckItem::new());
    incident.children.insert("Location".to_string(), CheckItem::new());
    incident.children.insert("Description".to_string(), CheckItem::new());
    incident.children.insert("Police Report Number".to_string(), CheckItem::new());
    checklist.insert("Incident Tab".to_string(), incident);
    
    // Injuries Tab
    let mut injuries = CheckItem::new();
    injuries.children.insert("Primary Injuries".to_string(), CheckItem::new());
    injuries.children.insert("Treatment Facilities".to_string(), CheckItem::new());
    injuries.children.insert("Current Treatment Status".to_string(), CheckItem::new());
    injuries.children.insert("Pre-existing Conditions".to_string(), CheckItem::new());
    checklist.insert("Injuries Tab".to_string(), injuries);
    
    // Health Insurance Tab
    let mut health = CheckItem::new();
    health.children.insert("Insurance Provider".to_string(), CheckItem::new());
    health.children.insert("Policy Number".to_string(), CheckItem::new());
    health.children.insert("Group Number".to_string(), CheckItem::new());
    health.children.insert("Coverage Details".to_string(), CheckItem::new());
    checklist.insert("Health Insurance Tab".to_string(), health);
    
    // Employment Tab
    let mut employment = CheckItem::new();
    employment.children.insert("Current Employer".to_string(), CheckItem::new());
    employment.children.insert("Job Title".to_string(), CheckItem::new());
    employment.children.insert("Work Status".to_string(), CheckItem::new());
    employment.children.insert("Lost Wages Documentation".to_string(), CheckItem::new());
    checklist.insert("Employment Tab".to_string(), employment);

    checklist
}