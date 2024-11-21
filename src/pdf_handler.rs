use lopdf::Document;
use std::path::Path;
use image::{RgbaImage, ImageBuffer, Rgba};
use std::fs;

// Define the page ranges for each section
#[derive(Clone)]
pub struct SectionDefinition {
    pub title: String,
    pub start_page: u32,
    pub end_page: u32,
    pub region: Option<PageRegion>,
}

#[derive(Clone)]
pub struct PageRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub struct PDFSection {
    pub title: String,
    pub page: u32,
    pub image: Option<RgbaImage>,
}

pub struct PDFHandler {
    document: Option<Document>,
    sections: Vec<PDFSection>,
    current_path: std::path::PathBuf,
}

impl PDFHandler {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            document: None,
            sections: Vec::new(),
            current_path: std::path::PathBuf::new(),
        })
    }

    pub fn render_page(&self, page_num: u32, region: Option<&PageRegion>) -> Option<RgbaImage> {
        if let Some(doc) = &self.document {
            println!("Rendering page {}", page_num); // Debug print

            let width = 612;  // Standard US Letter width at 72 DPI
            let height = 792; // Standard US Letter height at 72 DPI
            
            let (final_width, final_height) = if let Some(region) = region {
                (region.width as u32, region.height as u32)
            } else {
                (width, height)
            };

            // Create a new image buffer
            let mut img: RgbaImage = ImageBuffer::new(final_width, final_height);
            
            // Fill with light gray background to make it visible
            for pixel in img.pixels_mut() {
                *pixel = Rgba([240, 240, 240, 255]);
            }

            // Add some visual indication that we're showing a PDF page
            // Draw a border
            for x in 0..final_width {
                img.put_pixel(x, 0, Rgba([100, 100, 100, 255]));
                img.put_pixel(x, final_height - 1, Rgba([100, 100, 100, 255]));
            }
            for y in 0..final_height {
                img.put_pixel(0, y, Rgba([100, 100, 100, 255]));
                img.put_pixel(final_width - 1, y, Rgba([100, 100, 100, 255]));
            }

            // Draw diagonal lines to make it clear this is a placeholder
            for i in 0..final_width.min(final_height) {
                img.put_pixel(i, i, Rgba([180, 180, 180, 255]));
                if i < final_width && (final_height - i - 1) < final_height {
                    img.put_pixel(i, final_height - i - 1, Rgba([180, 180, 180, 255]));
                }
            }

            Some(img)
        } else {
            println!("No document loaded");
            None
        }
    }

    pub fn load_pdf<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let path_buf = path.as_ref().to_path_buf();
        self.current_path = path_buf.clone();
        
        // Load the PDF document
        match Document::load(&path_buf) {
            Ok(doc) => {
                println!("PDF loaded successfully"); // Debug print
                self.document = Some(doc);
                
                // Create sections based on predefined definitions
                self.sections = Self::get_section_definitions()
                    .into_iter()
                    .map(|def| {
                        println!("Creating section: {}", def.title); // Debug print
                        PDFSection {
                            title: def.title,
                            page: def.start_page,
                            image: self.render_page(def.start_page, def.region.as_ref()),
                        }
                    })
                    .collect();
                Ok(())
            },
            Err(e) => {
                println!("Failed to load PDF: {}", e); // Debug print
                Err(e.to_string())
            }
        }
    }

    fn get_section_definitions() -> Vec<SectionDefinition> {
        vec![
            SectionDefinition {
                title: "Home Tab".to_string(),
                start_page: 1,
                end_page: 1,
                region: None,  // Show full page for Home Tab
            },
            SectionDefinition {
                title: "Client Tab - Index Information".to_string(),
                start_page: 2,
                end_page: 2,
                region: Some(PageRegion {
                    x: 50.0,
                    y: 150.0,
                    width: 500.0,
                    height: 300.0,
                }),
            },
            // Add more sections as needed
        ]
    }

    pub fn get_sections(&self) -> &[PDFSection] {
        &self.sections
    }
}
