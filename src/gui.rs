use std::{ffi::{CStr, CString, c_char, c_int, c_void}, path::PathBuf, sync::{Mutex, OnceLock}};

#[repr(C)] struct GtkApplication(c_void);
#[repr(C)] struct GtkWidget(c_void);

#[link(name = "gtk-4")]
unsafe extern "C" {
    fn gtk_application_new(app_id: *const c_char, flags: c_int) -> *mut GtkApplication;
    fn gtk_application_window_new(app: *mut GtkApplication) -> *mut GtkWidget;
    fn gtk_window_new() -> *mut GtkWidget;
    fn gtk_window_set_title(window: *mut GtkWidget, title: *const c_char);
    fn gtk_window_set_default_size(window: *mut GtkWidget, width: c_int, height: c_int);
    fn gtk_window_set_child(window: *mut GtkWidget, child: *mut GtkWidget);
    fn gtk_window_set_modal(window: *mut GtkWidget, modal: c_int);
    fn gtk_window_set_transient_for(window: *mut GtkWidget, parent: *mut GtkWidget);
    fn gtk_window_destroy(window: *mut GtkWidget);
    fn gtk_box_new(orientation: c_int, spacing: c_int) -> *mut GtkWidget;
    fn gtk_box_append(container: *mut GtkWidget, child: *mut GtkWidget);
    fn gtk_label_new(text: *const c_char) -> *mut GtkWidget;
    fn gtk_label_set_text(label: *mut GtkWidget, text: *const c_char);
    fn gtk_entry_new() -> *mut GtkWidget;
    fn gtk_editable_set_text(entry: *mut GtkWidget, text: *const c_char);
    fn gtk_editable_get_text(entry: *mut GtkWidget) -> *const c_char;
    fn gtk_button_new_with_label(text: *const c_char) -> *mut GtkWidget;
    fn gtk_spin_button_new_with_range(min: f64, max: f64, step: f64) -> *mut GtkWidget;
    fn gtk_spin_button_get_value_as_int(spin: *mut GtkWidget) -> c_int;
    fn gtk_drawing_area_new() -> *mut GtkWidget;
    fn gtk_drawing_area_set_content_width(area: *mut GtkWidget, width: c_int);
    fn gtk_drawing_area_set_content_height(area: *mut GtkWidget, height: c_int);
    fn gtk_drawing_area_set_draw_func(area: *mut GtkWidget, draw: unsafe extern "C" fn(*mut GtkWidget, *mut Cairo, c_int, c_int, *mut c_void), data: *mut c_void, destroy: Option<unsafe extern "C" fn(*mut c_void)>);
    fn gtk_widget_set_margin_top(widget: *mut GtkWidget, margin: c_int);
    fn gtk_widget_set_margin_bottom(widget: *mut GtkWidget, margin: c_int);
    fn gtk_widget_set_margin_start(widget: *mut GtkWidget, margin: c_int);
    fn gtk_widget_set_margin_end(widget: *mut GtkWidget, margin: c_int);
    fn gtk_widget_add_css_class(widget: *mut GtkWidget, name: *const c_char);
    fn gtk_window_present(window: *mut GtkWidget);
    fn gtk_widget_queue_draw(widget: *mut GtkWidget);
}

#[repr(C)] struct Cairo(c_void);
#[link(name = "cairo")]
unsafe extern "C" {
    fn cairo_set_source_rgb(cr: *mut Cairo, r: f64, g: f64, b: f64);
    fn cairo_set_source_rgba(cr: *mut Cairo, r: f64, g: f64, b: f64, a: f64);
    fn cairo_paint(cr: *mut Cairo);
    fn cairo_rectangle(cr: *mut Cairo, x: f64, y: f64, width: f64, height: f64);
    fn cairo_fill(cr: *mut Cairo);
    fn cairo_stroke(cr: *mut Cairo);
    fn cairo_set_line_width(cr: *mut Cairo, width: f64);
    fn cairo_move_to(cr: *mut Cairo, x: f64, y: f64);
    fn cairo_line_to(cr: *mut Cairo, x: f64, y: f64);
    fn cairo_select_font_face(cr: *mut Cairo, family: *const c_char, slant: c_int, weight: c_int);
    fn cairo_set_font_size(cr: *mut Cairo, size: f64);
    fn cairo_show_text(cr: *mut Cairo, text: *const c_char);
}

#[link(name = "gio-2.0")]
unsafe extern "C" { fn g_application_run(app: *mut GtkApplication, argc: c_int, argv: *mut *mut c_char) -> c_int; }
#[link(name = "gobject-2.0")]
unsafe extern "C" {
    fn g_signal_connect_data(instance: *mut c_void, signal: *const c_char, callback: unsafe extern "C" fn(), data: *mut c_void, destroy: Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>, flags: c_int) -> u64;
}

struct Ui { window: *mut GtkWidget, entry: *mut GtkWidget, pages: *mut GtkWidget, status: *mut GtkWidget, preview: *mut GtkWidget }
struct Pending { ui: *mut Ui, dialog: *mut GtkWidget, path: String, pages: usize }
static PREVIEW_PAGE: OnceLock<Mutex<Option<crate::PageDesign>>> = OnceLock::new();

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let app_id = CString::new("org.paperseal.generator")?;
    unsafe {
        let app = gtk_application_new(app_id.as_ptr(), 0);
        let signal = CString::new("activate")?;
        g_signal_connect_data(app.cast(), signal.as_ptr(), std::mem::transmute::<unsafe extern "C" fn(*mut GtkApplication, *mut c_void), unsafe extern "C" fn()>(activate), std::ptr::null_mut(), None, 0);
        g_application_run(app, 0, std::ptr::null_mut());
    }
    Ok(())
}

unsafe extern "C" fn activate(app: *mut GtkApplication, _: *mut c_void) {
    unsafe {
        let window = gtk_application_window_new(app);
        let title = c("Paperseal — A4 page generator");
        gtk_window_set_title(window, title.as_ptr());
        gtk_window_set_default_size(window, 560, 760);
        let panel = gtk_box_new(1, 12);
        for set_margin in [gtk_widget_set_margin_top, gtk_widget_set_margin_bottom, gtk_widget_set_margin_start, gtk_widget_set_margin_end] { set_margin(panel, 28); }
        let heading = gtk_label_new(c("Create a printable page").as_ptr());
        gtk_widget_add_css_class(heading, c("title-1").as_ptr());
        gtk_box_append(panel, heading);
        gtk_box_append(panel, gtk_label_new(c("A4 • Fine-line background • Unique page ID • Stamp area").as_ptr()));
        let preview = gtk_drawing_area_new();
        gtk_drawing_area_set_content_width(preview, 300);
        gtk_drawing_area_set_content_height(preview, 420);
        gtk_drawing_area_set_draw_func(preview, draw_preview, std::ptr::null_mut(), None);
        gtk_box_append(panel, preview);
        gtk_box_append(panel, gtk_label_new(c("Choose where to save the PDF").as_ptr()));
        let entry = gtk_entry_new();
        gtk_editable_set_text(entry, c("paperseal-page.pdf").as_ptr());
        gtk_box_append(panel, entry);
        gtk_box_append(panel, gtk_label_new(c("Number of pages (1–100)").as_ptr()));
        let pages = gtk_spin_button_new_with_range(1.0, 100.0, 1.0);
        gtk_box_append(panel, pages);
        let button = gtk_button_new_with_label(c("Generate A4 PDF").as_ptr());
        gtk_box_append(panel, button);
        let status = gtk_label_new(c("Ready. Randomness comes from Linux's secure system random source.").as_ptr());
        gtk_box_append(panel, status);
        gtk_box_append(panel, gtk_label_new(c("Print at 100% / Actual Size. A physical stamp can be added in the marked box.").as_ptr()));
        gtk_window_set_child(window, panel);
        let ui = Box::into_raw(Box::new(Ui { window, entry, pages, status, preview }));
        g_signal_connect_data(button.cast(), c("clicked").as_ptr(), std::mem::transmute::<unsafe extern "C" fn(*mut GtkWidget, *mut c_void), unsafe extern "C" fn()>(generate_clicked), ui.cast(), Some(drop_ui), 0);
        gtk_window_present(window);
    }
}

unsafe extern "C" fn draw_preview(_: *mut GtkWidget, cr: *mut Cairo, width: c_int, height: c_int, _: *mut c_void) {
    unsafe {
        cairo_set_source_rgb(cr, 0.90, 0.91, 0.93); cairo_paint(cr);
        let pad = 12.0;
        let scale = ((width as f64 - 2.0 * pad) / 595.0).min((height as f64 - 2.0 * pad) / 842.0);
        let w = 595.0 * scale; let h = 842.0 * scale;
        let ox = (width as f64 - w) / 2.0; let oy = (height as f64 - h) / 2.0;
        cairo_set_source_rgb(cr, 1.0, 1.0, 1.0); cairo_rectangle(cr, ox, oy, w, h); cairo_fill(cr);
        cairo_set_source_rgb(cr, 0.48, 0.50, 0.54); cairo_set_line_width(cr, 0.65);
        if let Some(page) = PREVIEW_PAGE.get_or_init(|| Mutex::new(None)).lock().unwrap().as_ref() {
            for line in &page.lines {
                let gray = line.gray;
                cairo_set_source_rgba(cr, gray, gray, gray, 0.30);
                cairo_set_line_width(cr, line.width * scale);
                cairo_move_to(cr, ox + line.x1 * scale, oy + (842.0 - line.y1) * scale);
                cairo_line_to(cr, ox + line.x2 * scale, oy + (842.0 - line.y2) * scale);
                cairo_stroke(cr);
            }
        }
        cairo_set_source_rgb(cr, 1.0, 1.0, 1.0);
        cairo_rectangle(cr, ox + 30.0 * scale, oy + 18.0 * scale, 300.0 * scale, 22.0 * scale); cairo_fill(cr);
        cairo_rectangle(cr, ox + 370.0 * scale, oy + (842.0 - 140.0) * scale, 190.0 * scale, 92.0 * scale); cairo_fill(cr);
        cairo_set_source_rgb(cr, 0.0, 0.0, 0.0);
        cairo_select_font_face(cr, c("sans").as_ptr(), 0, 0); cairo_set_font_size(cr, 10.0 * scale);
        cairo_move_to(cr, ox + 34.0 * scale, oy + 32.0 * scale); cairo_show_text(cr, c("PAPERSEAL").as_ptr());
        let id = PREVIEW_PAGE.get_or_init(|| Mutex::new(None)).lock().unwrap().as_ref().map(|p| p.id.clone()).unwrap_or_else(|| "Preview will appear here".into());
        cairo_move_to(cr, ox + 415.0 * scale, oy + 32.0 * scale); cairo_show_text(cr, c(&format!("ID: {id}")).as_ptr());
        cairo_set_source_rgb(cr, 0.0, 0.0, 0.0); cairo_set_line_width(cr, 0.8 * scale);
        cairo_rectangle(cr, ox + 370.0 * scale, oy + (842.0 - 140.0) * scale, 190.0 * scale, 92.0 * scale);
        cairo_stroke(cr);
        cairo_select_font_face(cr, c("sans").as_ptr(), 0, 0); cairo_set_font_size(cr, 9.0 * scale);
        cairo_move_to(cr, ox + 380.0 * scale, oy + (842.0 - 123.0) * scale); cairo_show_text(cr, c("STAMP / VALIDATION").as_ptr());
        cairo_set_source_rgb(cr, 0.5, 0.5, 0.5); cairo_set_line_width(cr, 0.4 * scale);
        for y in [63.0, 75.0, 87.0] {
            cairo_move_to(cr, ox + 380.0 * scale, oy + (842.0 - y) * scale);
            cairo_line_to(cr, ox + 550.0 * scale, oy + (842.0 - y) * scale); cairo_stroke(cr);
        }
    }
}

unsafe extern "C" fn generate_clicked(_: *mut GtkWidget, data: *mut c_void) {
    let ui = unsafe { &*(data as *mut Ui) };
    let raw = unsafe { gtk_editable_get_text(ui.entry) };
    if raw.is_null() { set_status(ui.status, "Enter a PDF filename."); return; }
    let input = unsafe { CStr::from_ptr(raw) }.to_string_lossy().trim().to_owned();
    if input.is_empty() { set_status(ui.status, "Enter a PDF filename."); return; }
    let path = PathBuf::from(&input);
    let output = if path.extension().is_none() { path.with_extension("pdf") } else { path };
    let page_count = unsafe { gtk_spin_button_get_value_as_int(ui.pages) }.max(1) as usize;
    let output = output.to_string_lossy().into_owned();
    if PathBuf::from(&output).exists() {
        unsafe { show_overwrite_confirmation(ui, output, page_count); }
        return;
    }
    generate_to_path(ui, &output, page_count);
}

fn generate_to_path(ui: &Ui, output: &str, page_count: usize) {
    match crate::generate_pages_with_preview(output, page_count) {
        Ok((ids, preview)) => {
            *PREVIEW_PAGE.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(preview);
            unsafe { gtk_widget_queue_draw(ui.preview); }
            set_status(ui.status, &format!("Created {} pages at {}   •   IDs: {}", ids.len(), output, ids.join(", ")))
        },
        Err(error) => set_status(ui.status, &format!("Could not create PDF: {error}")),
    }
}

unsafe fn show_overwrite_confirmation(ui: &Ui, path: String, pages: usize) {
    unsafe {
        let dialog = gtk_window_new();
        // Use a regular GTK window so this works on Arch's GTK 4.8 without newer AlertDialog APIs.
        gtk_window_set_title(dialog, c("Confirm overwrite").as_ptr());
        gtk_window_set_default_size(dialog, 430, 150);
        gtk_window_set_transient_for(dialog, ui.window);
        gtk_window_set_modal(dialog, 1);
        let box_widget = gtk_box_new(1, 12);
        for margin in [gtk_widget_set_margin_top, gtk_widget_set_margin_bottom, gtk_widget_set_margin_start, gtk_widget_set_margin_end] { margin(box_widget, 18); }
        gtk_box_append(box_widget, gtk_label_new(c(&format!("A file named ‘{path}’ already exists. Replace it?\nThe existing file will be overwritten.")).as_ptr()));
        let buttons = gtk_box_new(0, 8);
        let cancel = gtk_button_new_with_label(c("Cancel").as_ptr());
        let overwrite = gtk_button_new_with_label(c("Overwrite").as_ptr());
        gtk_box_append(buttons, cancel); gtk_box_append(buttons, overwrite); gtk_box_append(box_widget, buttons);
        gtk_window_set_child(dialog, box_widget);
        for (button, callback) in [(cancel, cancel_overwrite as unsafe extern "C" fn(*mut GtkWidget, *mut c_void)), (overwrite, confirm_overwrite as unsafe extern "C" fn(*mut GtkWidget, *mut c_void))] {
            let pending = Box::into_raw(Box::new(Pending { ui: ui as *const Ui as *mut Ui, dialog, path: path.clone(), pages }));
            g_signal_connect_data(button.cast(), c("clicked").as_ptr(), std::mem::transmute::<unsafe extern "C" fn(*mut GtkWidget, *mut c_void), unsafe extern "C" fn()>(callback), pending.cast(), Some(drop_pending), 0);
        }
        gtk_window_present(dialog);
    }
}

unsafe extern "C" fn confirm_overwrite(_: *mut GtkWidget, data: *mut c_void) {
    let pending = unsafe { &*(data as *mut Pending) };
    generate_to_path(unsafe { &*pending.ui }, &pending.path, pending.pages);
    unsafe { gtk_window_destroy(pending.dialog); }
}
unsafe extern "C" fn cancel_overwrite(_: *mut GtkWidget, data: *mut c_void) {
    let pending = unsafe { &*(data as *mut Pending) };
    set_status(unsafe { &*pending.ui }.status, "Overwrite cancelled; existing file was kept.");
    unsafe { gtk_window_destroy(pending.dialog); }
}
unsafe extern "C" fn drop_pending(data: *mut c_void, _: *mut c_void) { drop(unsafe { Box::from_raw(data as *mut Pending) }); }

unsafe extern "C" fn drop_ui(data: *mut c_void, _: *mut c_void) { drop(unsafe { Box::from_raw(data as *mut Ui) }); }
fn set_status(label: *mut GtkWidget, text: &str) { unsafe { gtk_label_set_text(label, c(text).as_ptr()); } }
fn c(text: &str) -> CString { CString::new(text).unwrap_or_else(|_| CString::new("Invalid text").unwrap()) }
