use std::{fs::File, io::{self, Read, Write}, path::Path};

const W: f64 = 595.0;
const H: f64 = 842.0;

#[derive(Clone)]
pub struct Segment { pub x1: f64, pub y1: f64, pub x2: f64, pub y2: f64, pub width: f64, pub gray: f64 }
#[derive(Clone)]
pub struct PageDesign { pub id: String, pub lines: Vec<Segment> }

struct Rng { file: File }
impl Rng {
    fn new() -> io::Result<Self> { Ok(Self { file: File::open("/dev/urandom")? }) }
    fn u32(&mut self) -> io::Result<u32> {
        let mut b = [0u8; 4]; self.file.read_exact(&mut b)?; Ok(u32::from_ne_bytes(b))
    }
    fn unit(&mut self) -> io::Result<f64> { Ok(self.u32()? as f64 / u32::MAX as f64) }
}

pub mod gui;

pub fn generate(dest: &str) -> Result<String, String> {
    generate_pages(dest, 1).map(|mut ids| ids.remove(0))
}

pub fn generate_pages(dest: &str, page_count: usize) -> Result<Vec<String>, String> {
    generate_pages_with_preview(dest, page_count).map(|(ids, _)| ids)
}

pub fn generate_pages_with_preview(dest: &str, page_count: usize) -> Result<(Vec<String>, PageDesign), String> {
    if Path::new(dest).extension().and_then(|s| s.to_str()) != Some("pdf") {
        return Err("Choose a filename ending in .pdf".into());
    }
    if !(1..=100).contains(&page_count) { return Err("Page count must be between 1 and 100".into()); }
    let mut rng = Rng::new().map_err(|e| e.to_string())?;
    let mut ids = Vec::with_capacity(page_count);
    let mut streams = Vec::with_capacity(page_count);
    let mut preview = None;
    for _ in 0..page_count {
        let mut code = String::new();
        for _ in 0..12 { code.push_str(&format!("{:X}", rng.u32().map_err(|e| e.to_string())? & 0xF)); }
        let page = make_page_design(&mut rng, &code).map_err(|e| e.to_string())?;
        streams.push(page_stream(&page));
        if preview.is_none() { preview = Some(page.clone()); }
        ids.push(code);
    }
    let pdf = make_pdf(streams).map_err(|e| e.to_string())?;
    let mut f = File::create(dest).map_err(|e| e.to_string())?;
    f.write_all(&pdf).map_err(|e| e.to_string())?;
    Ok((ids, preview.expect("page count validated as positive")))
}

fn make_page_design(rng: &mut Rng, code: &str) -> io::Result<PageDesign> {
    let mut lines = Vec::new();
    // Fine, gently wandering horizontal lines: printable on ordinary office printers.
    let mut y = 54.0;
    while y < 790.0 {
        let phase = rng.unit()? * 6.28318;
        let amp = 1.0 + rng.unit()? * 2.2;
        let slope = (rng.unit()? - 0.5) * 0.006;
        let width = 1.55;
        let mut x: f64 = 38.0;
        let mut py = y + amp * phase.sin();
        while x < 557.0 {
            let nx = (x + 7.0).min(557.0);
            let ny = y + amp * ((nx / 29.0) + phase).sin() + (nx - 297.5) * slope;
            lines.push(Segment { x1: x, y1: py, x2: nx, y2: ny, width, gray: 0.0 });
            x = nx; py = ny;
        }
        y += 8.0 + rng.unit()? * 1.4;
    }
    // Sparse crossing arcs add a distinct, irregular texture without dense toner coverage.
    for i in 0..30 {
        let x0 = 42.0 + rng.unit()? * 500.0;
        let y0 = 65.0 + rng.unit()? * 710.0;
        let dx = 15.0 + rng.unit()? * 45.0;
        let dy = (rng.unit()? - 0.5) * 10.0;
        lines.push(Segment { x1: x0, y1: y0, x2: (x0 + dx).min(557.0), y2: (y0 + dy).clamp(44.0, 798.0), width: if i % 3 == 0 { 1.05 } else { 0.90 }, gray: if i % 3 == 0 { 0.10 } else { 0.22 } });
    }
    Ok(PageDesign { id: code.to_string(), lines })
}

fn page_stream(page: &PageDesign) -> Vec<u8> {
    let mut s = String::from("q\n/GS1 gs\n");
    for line in &page.lines {
        s.push_str(&format!("{:.2} G {:.2} w\n{:.2} {:.2} m {:.2} {:.2} l S\n", line.gray, line.width, line.x1, line.y1, line.x2, line.y2));
    }
    s.push_str("Q\n");
    // Quiet header and stamp panel are deliberately left clear of the background.
    s.push_str("1 g 30 802 300 22 re f\n0 g\nBT /F1 10 Tf 34 810 Td (PAPERSEAL) Tj ET\n");
    s.push_str(&format!("BT /F1 10 Tf 415 810 Td (ID: {}) Tj ET\n", page.id));
    s.push_str("1 g 370 48 190 92 re f\n0 G 0.8 w 370 48 190 92 re S\nBT /F1 9 Tf 380 123 Td (STAMP / VALIDATION) Tj ET\n");
    s.push_str("0.5 G 0.4 w 380 63 m 550 63 l S\n380 75 m 550 75 l S\n380 87 m 550 87 l S\n");
    s.into_bytes()
}

fn make_pdf(streams: Vec<Vec<u8>>) -> io::Result<Vec<u8>> {
    let mut pdf = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = vec![0usize];
    let mut objects: Vec<Vec<u8>> = vec![
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        Vec::new(),
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec(),
        b"<< /Type /ExtGState /CA 0.3 /ca 0.3 >>".to_vec(),
    ];
    let mut kids = String::new();
    for (index, stream) in streams.into_iter().enumerate() {
        let page_obj = 5 + index * 2;
        let content_obj = page_obj + 1;
        kids.push_str(&format!("{page_obj} 0 R "));
        objects.push(format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {W} {H}] /Resources << /Font << /F1 3 0 R >> /ExtGState << /GS1 4 0 R >> >> /Contents {content_obj} 0 R >>").into_bytes());
        objects.push([format!("<< /Length {} >>\nstream\n", stream.len()).into_bytes(), stream, b"endstream".to_vec()].concat());
    }
    objects[1] = format!("<< /Type /Pages /Kids [{kids}] /Count {} >>", objects.len().saturating_sub(4) / 2).into_bytes();
    for (i, obj) in objects.iter().enumerate() {
        offsets.push(pdf.len()); pdf.extend_from_slice(format!("{} 0 obj\n", i+1).as_bytes());
        pdf.extend_from_slice(obj); pdf.extend_from_slice(b"\nendobj\n");
    }
    let xref = pdf.len(); pdf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len()+1).as_bytes());
    for off in offsets.iter().skip(1) { pdf.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes()); }
    pdf.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len()+1).as_bytes());
    Ok(pdf)
}
