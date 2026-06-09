use crate::error::AppError;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::io::Write as IoWrite;
use usvg::tiny_skia_path::PathSegment;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FontFormat {
    Ttf,
    Woff,
    Woff2,
    Eot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontExportOptions {
    pub font_name: String,
    pub formats: Vec<FontFormat>,
    pub include_css: bool,
    pub include_demo: bool,
    pub unicode_start: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlyphEntry {
    pub icon_name: String,
    pub unicode: char,
    pub svg_path_data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontExportResult {
    pub files: Vec<(String, Vec<u8>)>,
}

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct GlyphPoint {
    x: f32,
    y: f32,
    on_curve: bool,
}

#[derive(Debug, Clone)]
struct GlyphData {
    contours: Vec<Vec<GlyphPoint>>,
    advance_width: u16,
    x_min: i16,
    y_min: i16,
    x_max: i16,
    y_max: i16,
}

#[derive(Debug, Clone)]
enum Seg {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    QuadTo(f32, f32, f32, f32),
    Close,
}

// ---------------------------------------------------------------------------
// Binary helpers
// ---------------------------------------------------------------------------

fn put_u8(buf: &mut Vec<u8>, v: u8) {
    buf.push(v);
}

fn put_u16(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_be_bytes());
}

fn put_i16(buf: &mut Vec<u8>, v: i16) {
    buf.extend_from_slice(&v.to_be_bytes());
}

fn put_u32(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_be_bytes());
}

fn put_i64(buf: &mut Vec<u8>, v: i64) {
    buf.extend_from_slice(&v.to_be_bytes());
}

fn pad4(buf: &mut Vec<u8>) {
    while !buf.len().is_multiple_of(4) {
        buf.push(0);
    }
}

fn checksum(data: &[u8]) -> u32 {
    let mut sum: u32 = 0;
    // Pad to 4-byte boundary
    let len = (data.len() + 3) & !3;
    for i in (0..len).step_by(4) {
        let b0 = data.get(i).copied().unwrap_or(0);
        let b1 = data.get(i + 1).copied().unwrap_or(0);
        let b2 = data.get(i + 2).copied().unwrap_or(0);
        let b3 = data.get(i + 3).copied().unwrap_or(0);
        sum = sum.wrapping_add(u32::from_be_bytes([b0, b1, b2, b3]));
    }
    sum
}

// ---------------------------------------------------------------------------
// SVG → Glyph conversion
// ---------------------------------------------------------------------------

fn empty_glyph(em: u16) -> GlyphData {
    GlyphData {
        contours: Vec::new(),
        advance_width: em,
        x_min: 0,
        y_min: 0,
        x_max: 0,
        y_max: 0,
    }
}

fn contours_bbox(contours: &[Vec<GlyphPoint>]) -> (i16, i16, i16, i16) {
    let mut xn = i16::MAX;
    let mut yn = i16::MAX;
    let mut xx = i16::MIN;
    let mut yx = i16::MIN;
    for c in contours {
        for p in c {
            let ix = p.x.round() as i16;
            let iy = p.y.round() as i16;
            xn = xn.min(ix);
            yn = yn.min(iy);
            xx = xx.max(ix);
            yx = yx.max(iy);
        }
    }
    if xn == i16::MAX { (0, 0, 0, 0) } else { (xn, yn, xx, yx) }
}

fn collect_segments(
    group: &usvg::Group,
    scale: f32,
    ox: f32,
    oy: f32,
    em: f32,
    out: &mut Vec<Seg>,
    pos: &mut (f32, f32),
) {
    for child in group.children() {
        match child {
            usvg::Node::Group(g) => collect_segments(g, scale, ox, oy, em, out, pos),
            usvg::Node::Path(p) => {
                for seg in p.data().segments() {
                    match seg {
                        PathSegment::MoveTo(pt) => {
                            let x = pt.x * scale + ox;
                            let y = em - (pt.y * scale + oy);
                            out.push(Seg::MoveTo(x, y));
                            *pos = (x, y);
                        }
                        PathSegment::LineTo(pt) => {
                            let x = pt.x * scale + ox;
                            let y = em - (pt.y * scale + oy);
                            out.push(Seg::LineTo(x, y));
                            *pos = (x, y);
                        }
                        PathSegment::QuadTo(p1, p2) => {
                            let x1 = p1.x * scale + ox;
                            let y1 = em - (p1.y * scale + oy);
                            let x2 = p2.x * scale + ox;
                            let y2 = em - (p2.y * scale + oy);
                            out.push(Seg::QuadTo(x1, y1, x2, y2));
                            *pos = (x2, y2);
                        }
                        PathSegment::CubicTo(p1, p2, p3) => {
                            let (lx, ly) = *pos;
                            let cp1x = p1.x * scale + ox;
                            let cp1y = em - (p1.y * scale + oy);
                            let cp2x = p2.x * scale + ox;
                            let cp2y = em - (p2.y * scale + oy);
                            let ex = p3.x * scale + ox;
                            let ey = em - (p3.y * scale + oy);

                            // de Casteljau split at t=0.5
                            let p01x = (lx + cp1x) / 2.0;
                            let p01y = (ly + cp1y) / 2.0;
                            let p12x = (cp1x + cp2x) / 2.0;
                            let p12y = (cp1y + cp2y) / 2.0;
                            let p23x = (cp2x + ex) / 2.0;
                            let p23y = (cp2y + ey) / 2.0;
                            let p012x = (p01x + p12x) / 2.0;
                            let p012y = (p01y + p12y) / 2.0;
                            let p123x = (p12x + p23x) / 2.0;
                            let p123y = (p12y + p23y) / 2.0;
                            let mx = (p012x + p123x) / 2.0;
                            let my = (p012y + p123y) / 2.0;

                            let q1x = 2.0 * p012x - (lx + mx) / 2.0;
                            let q1y = 2.0 * p012y - (ly + my) / 2.0;
                            out.push(Seg::QuadTo(q1x, q1y, mx, my));

                            let q2x = 2.0 * p123x - (mx + ex) / 2.0;
                            let q2y = 2.0 * p123y - (my + ey) / 2.0;
                            out.push(Seg::QuadTo(q2x, q2y, ex, ey));

                            *pos = (ex, ey);
                        }
                        PathSegment::Close => {
                            out.push(Seg::Close);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn segments_to_contours(segs: &[Seg]) -> Vec<Vec<GlyphPoint>> {
    let mut all: Vec<Vec<GlyphPoint>> = Vec::new();
    let mut cur: Vec<GlyphPoint> = Vec::new();
    for s in segs {
        match s {
            Seg::MoveTo(x, y) => {
                if !cur.is_empty() {
                    all.push(std::mem::take(&mut cur));
                }
                cur.push(GlyphPoint { x: *x, y: *y, on_curve: true });
            }
            Seg::LineTo(x, y) => {
                cur.push(GlyphPoint { x: *x, y: *y, on_curve: true });
            }
            Seg::QuadTo(x1, y1, x2, y2) => {
                cur.push(GlyphPoint { x: *x1, y: *y1, on_curve: false });
                cur.push(GlyphPoint { x: *x2, y: *y2, on_curve: true });
            }
            Seg::Close => {
                if !cur.is_empty() {
                    all.push(std::mem::take(&mut cur));
                }
            }
        }
    }
    if !cur.is_empty() {
        all.push(cur);
    }
    all
}

fn svg_to_glyph(svg_str: &str, em: u16) -> Result<GlyphData, AppError> {
    if svg_str.trim().is_empty() {
        return Ok(empty_glyph(em));
    }
    let opts = crate::engine::renderer::get_options();
    let tree = usvg::Tree::from_str(svg_str, opts)
        .map_err(|e| AppError::ExportError(format!("SVG parse error: {}", e)))?;

    let sz = tree.size();
    let sw = sz.width();
    let sh = sz.height();
    if sw <= 0.0 || sh <= 0.0 {
        return Ok(empty_glyph(em));
    }

    let em_f = em as f32;
    let scale = em_f / sw.max(sh);
    let ox = (em_f - sw * scale) / 2.0;
    let oy = (em_f - sh * scale) / 2.0;

    let mut segs: Vec<Seg> = Vec::new();
    let mut pos = (0.0f32, 0.0f32);
    collect_segments(tree.root(), scale, ox, oy, em_f, &mut segs, &mut pos);

    let contours = segments_to_contours(&segs);
    let bbox = contours_bbox(&contours);

    Ok(GlyphData {
        contours,
        advance_width: em,
        x_min: bbox.0,
        y_min: bbox.1,
        x_max: bbox.2,
        y_max: bbox.3,
    })
}

// ---------------------------------------------------------------------------
// TTF glyph encoding
// ---------------------------------------------------------------------------

fn encode_empty_glyph() -> Vec<u8> {
    let mut b = Vec::with_capacity(10);
    put_i16(&mut b, 0); // numberOfContours
    put_i16(&mut b, 0); // xMin
    put_i16(&mut b, 0); // yMin
    put_i16(&mut b, 0); // xMax
    put_i16(&mut b, 0); // yMax
    b
}

fn encode_simple_glyph(contours: &[Vec<GlyphPoint>]) -> Vec<u8> {
    if contours.is_empty() {
        return encode_empty_glyph();
    }
    let nc = contours.len() as i16;
    let mut pts: Vec<GlyphPoint> = Vec::new();
    let mut end_pts: Vec<u16> = Vec::new();
    for c in contours {
        pts.extend_from_slice(c);
        end_pts.push((pts.len() - 1) as u16);
    }
    let (xn, yn, xx, yx) = contours_bbox(contours);

    let mut b = Vec::new();
    put_i16(&mut b, nc);
    put_i16(&mut b, xn);
    put_i16(&mut b, yn);
    put_i16(&mut b, xx);
    put_i16(&mut b, yx);
    for &ep in &end_pts {
        put_u16(&mut b, ep);
    }
    put_u16(&mut b, 0); // instructionLength

    let mut flags: Vec<u8> = Vec::new();
    let mut xd: Vec<u8> = Vec::new();
    let mut yd: Vec<u8> = Vec::new();
    let mut px: i16 = 0;
    let mut py: i16 = 0;

    for p in &pts {
        let x = p.x.round() as i16;
        let y = p.y.round() as i16;
        let dx = x - px;
        let dy = y - py;
        px = x;
        py = y;

        let mut f: u8 = 0;
        if p.on_curve { f |= 0x01; }

        if dx == 0 {
            f |= 0x10;
        } else if (-255..=255).contains(&dx) {
            f |= 0x02;
            if dx > 0 { f |= 0x10; }
            xd.push(dx.unsigned_abs() as u8);
        } else {
            xd.extend_from_slice(&dx.to_be_bytes());
        }

        if dy == 0 {
            f |= 0x20;
        } else if (-255..=255).contains(&dy) {
            f |= 0x04;
            if dy > 0 { f |= 0x20; }
            yd.push(dy.unsigned_abs() as u8);
        } else {
            yd.extend_from_slice(&dy.to_be_bytes());
        }

        flags.push(f);
    }

    b.extend_from_slice(&flags);
    b.extend_from_slice(&xd);
    b.extend_from_slice(&yd);
    b
}

// ---------------------------------------------------------------------------
// TTF table builders
// ---------------------------------------------------------------------------

fn build_head(em: u16, bbox: (i16, i16, i16, i16)) -> Vec<u8> {
    let mut b = Vec::with_capacity(54);
    put_u32(&mut b, 0x00010000); // version
    put_u32(&mut b, 0x00010000); // fontRevision
    put_u32(&mut b, 0);          // checksumAdjust (placeholder)
    put_u32(&mut b, 0x5F0F3CF5); // magicNumber
    put_u16(&mut b, 0x000B);     // flags
    put_u16(&mut b, em);         // unitsPerEm
    put_i64(&mut b, 0);          // created
    put_i64(&mut b, 0);          // modified
    put_i16(&mut b, bbox.0);
    put_i16(&mut b, bbox.1);
    put_i16(&mut b, bbox.2);
    put_i16(&mut b, bbox.3);
    put_u16(&mut b, 0);  // macStyle
    put_u16(&mut b, 8);  // lowestRecPPEM
    put_i16(&mut b, 2);  // fontDirectionHint
    put_i16(&mut b, 0);  // indexToLocFormat (short)
    put_i16(&mut b, 0);  // glyphDataFormat
    b
}

fn build_hhea(em: u16, num_glyphs: u16) -> Vec<u8> {
    let mut b = Vec::with_capacity(36);
    put_u16(&mut b, 1);
    put_u16(&mut b, 0);
    put_i16(&mut b, em as i16); // ascender
    put_i16(&mut b, 0);         // descender
    put_i16(&mut b, 0);         // lineGap
    put_u16(&mut b, em);        // advanceWidthMax
    put_i16(&mut b, 0);
    put_i16(&mut b, 0);
    put_i16(&mut b, em as i16); // xMaxExtent
    put_i16(&mut b, 1);         // caretSlopeRise
    put_i16(&mut b, 0);
    put_i16(&mut b, 0);
    for _ in 0..4 { put_i16(&mut b, 0); } // reserved
    put_i16(&mut b, 0); // metricDataFormat
    put_u16(&mut b, num_glyphs);
    b
}

fn build_maxp(num_glyphs: u16, max_pts: u16, max_ctrs: u16) -> Vec<u8> {
    let mut b = Vec::with_capacity(32);
    put_u32(&mut b, 0x00010000);
    put_u16(&mut b, num_glyphs);
    put_u16(&mut b, max_pts);
    put_u16(&mut b, max_ctrs);
    for _ in 0..12 { put_u16(&mut b, 0); }
    b
}

fn build_os2(em: u16, first_char: u16, last_char: u16) -> Vec<u8> {
    let mut b = Vec::with_capacity(86);
    put_u16(&mut b, 1);              // version
    put_i16(&mut b, em as i16 / 2);  // xAvgCharWidth
    put_u16(&mut b, 400);            // usWeightClass
    put_u16(&mut b, 5);              // usWidthClass
    put_u16(&mut b, 0);              // fsType
    put_i16(&mut b, 650); put_i16(&mut b, 600); put_i16(&mut b, 0); put_i16(&mut b, 75);
    put_i16(&mut b, 650); put_i16(&mut b, 600); put_i16(&mut b, 0); put_i16(&mut b, 350);
    put_i16(&mut b, 50);  // yStrikeoutSize
    put_i16(&mut b, 300); // yStrikeoutPosition
    put_i16(&mut b, 0);   // sFamilyClass
    b.extend_from_slice(&[0u8; 10]); // panose
    b.extend_from_slice(&[0u8; 16]); // unicodeRange
    b.extend_from_slice(b"NONE");    // achVendID
    put_u16(&mut b, 0x0040);        // fsSelection (regular)
    put_u16(&mut b, first_char);
    put_u16(&mut b, last_char);
    put_i16(&mut b, em as i16); // sTypoAscender
    put_i16(&mut b, 0);         // sTypoDescender
    put_i16(&mut b, 0);         // sTypoLineGap
    put_u16(&mut b, em);        // usWinAscent
    put_u16(&mut b, 0);         // usWinDescent
    put_u32(&mut b, 1);         // ulCodePageRange1
    put_u32(&mut b, 0);         // ulCodePageRange2
    b
}

fn build_name(name: &str) -> Vec<u8> {
    let to_utf16 = |s: &str| -> Vec<u8> {
        let mut b = Vec::new();
        for ch in s.encode_utf16() { put_u16(&mut b, ch); }
        b
    };
    let ps_name = name.replace(' ', "-");
    let unique_id = format!("{}-Regular", name);
    let full_name = format!("{} Regular", name);
    let records: Vec<(u16, &str)> = vec![
        (0, name), (1, name), (2, "Regular"),
        (3, &unique_id),
        (4, &full_name),
        (5, "Version 1.0"), (6, &ps_name),
    ];
    let n = records.len() as u16;
    let str_off = 6 + n as usize * 12;

    let mut hdr = Vec::new();
    put_u16(&mut hdr, 0); // format
    put_u16(&mut hdr, n);
    put_u16(&mut hdr, str_off as u16);

    let mut sd = Vec::new();
    let mut off = 0u16;
    for (nid, text) in &records {
        let enc = to_utf16(text);
        put_u16(&mut hdr, 3);       // platformID
        put_u16(&mut hdr, 1);       // encodingID
        put_u16(&mut hdr, 0x0409);  // languageID
        put_u16(&mut hdr, *nid);
        put_u16(&mut hdr, enc.len() as u16);
        put_u16(&mut hdr, off);
        sd.extend_from_slice(&enc);
        off += enc.len() as u16;
    }
    hdr.extend_from_slice(&sd);
    hdr
}

fn build_cmap(glyphs: &[(char, GlyphData)]) -> Vec<u8> {
    let mut buf = Vec::new();
    put_u16(&mut buf, 0); // version
    put_u16(&mut buf, 1); // numTables
    put_u16(&mut buf, 3); // platformID
    put_u16(&mut buf, 1); // encodingID
    put_u32(&mut buf, 12); // subtable offset

    if glyphs.is_empty() {
        let seg = 1u16;
        put_u16(&mut buf, 4);
        put_u16(&mut buf, 32);
        put_u16(&mut buf, 0);
        put_u16(&mut buf, seg * 2);
        put_u16(&mut buf, 2);
        put_u16(&mut buf, 0);
        put_u16(&mut buf, 0);
        put_u16(&mut buf, 0xFFFF);
        put_u16(&mut buf, 0);
        put_u16(&mut buf, 0xFFFF);
        put_i16(&mut buf, 1);
        put_u16(&mut buf, 0);
        return buf;
    }

    let first = glyphs[0].0 as u16;
    let last = glyphs.last().unwrap().0 as u16;
    let delta = (1i32 - first as i32) as i16;
    let seg = 2u16;
    let sr = 4u16;
    let es = 1u16;

    put_u16(&mut buf, 4); // format
    // We'll backfill length
    let len_pos = buf.len();
    put_u16(&mut buf, 0); // placeholder
    put_u16(&mut buf, 0); // language
    put_u16(&mut buf, seg * 2);
    put_u16(&mut buf, sr);
    put_u16(&mut buf, es);
    put_u16(&mut buf, seg * 2 - sr);

    put_u16(&mut buf, last);
    put_u16(&mut buf, 0xFFFF);
    put_u16(&mut buf, 0); // reservedPad
    put_u16(&mut buf, first);
    put_u16(&mut buf, 0xFFFF);
    put_i16(&mut buf, delta);
    put_i16(&mut buf, 1);
    put_u16(&mut buf, 0);
    put_u16(&mut buf, 0);

    let total = buf.len() as u16;
    buf[len_pos] = (total >> 8) as u8;
    buf[len_pos + 1] = total as u8;
    buf
}

fn build_post() -> Vec<u8> {
    let mut b = Vec::with_capacity(32);
    put_u32(&mut b, 0x00030000);
    put_u32(&mut b, 0);
    put_i16(&mut b, -100);
    put_i16(&mut b, 50);
    put_u32(&mut b, 0);
    put_u32(&mut b, 0);
    put_u32(&mut b, 0);
    put_u32(&mut b, 0);
    put_u32(&mut b, 0);
    b
}

fn build_hmtx(glyphs: &[(char, GlyphData)], em: u16) -> Vec<u8> {
    let mut b = Vec::new();
    // .notdef
    put_u16(&mut b, em);
    put_i16(&mut b, 0);
    for (_, g) in glyphs {
        put_u16(&mut b, g.advance_width);
        put_i16(&mut b, g.x_min);
    }
    b
}

// ---------------------------------------------------------------------------
// TTF assembly
// ---------------------------------------------------------------------------

fn build_ttf(glyphs: &[(char, GlyphData)], name: &str, em: u16) -> Result<Vec<u8>, AppError> {
    let num_glyphs = glyphs.len() as u16 + 1;

    let mut bins: Vec<Vec<u8>> = Vec::new();
    bins.push(encode_empty_glyph());

    let mut max_pts = 0u16;
    let mut max_ctrs = 0u16;
    let mut bx = (i16::MAX, i16::MAX, i16::MIN, i16::MIN);

    for (_, g) in glyphs {
        let n: usize = g.contours.iter().map(|c| c.len()).sum();
        max_pts = max_pts.max(n as u16);
        max_ctrs = max_ctrs.max(g.contours.len() as u16);
        bx.0 = bx.0.min(g.x_min);
        bx.1 = bx.1.min(g.y_min);
        bx.2 = bx.2.max(g.x_max);
        bx.3 = bx.3.max(g.y_max);
        bins.push(encode_simple_glyph(&g.contours));
    }
    if bx.0 == i16::MAX {
        bx = (0, 0, em as i16, em as i16);
    }

    // glyf + loca
    let mut glyf = Vec::new();
    let mut loca_offs: Vec<u16> = Vec::new();
    for bin in &bins {
        if glyf.len() % 2 != 0 { glyf.push(0); }
        loca_offs.push((glyf.len() / 2) as u16);
        glyf.extend_from_slice(bin);
    }
    if glyf.len() % 2 != 0 { glyf.push(0); }
    loca_offs.push((glyf.len() / 2) as u16);

    let mut loca = Vec::new();
    for &o in &loca_offs { put_u16(&mut loca, o); }

    let fc = glyphs.first().map(|(c, _)| *c as u16).unwrap_or(0);
    let lc = glyphs.last().map(|(c, _)| *c as u16).unwrap_or(0);

    let tables: Vec<(&[u8; 4], Vec<u8>)> = vec![
        (b"head", build_head(em, bx)),
        (b"hhea", build_hhea(em, num_glyphs)),
        (b"maxp", build_maxp(num_glyphs, max_pts, max_ctrs)),
        (b"OS/2", build_os2(em, fc, lc)),
        (b"name", build_name(name)),
        (b"cmap", build_cmap(glyphs)),
        (b"post", build_post()),
        (b"glyf", glyf),
        (b"loca", loca),
        (b"hmtx", build_hmtx(glyphs, em)),
    ];

    Ok(assemble_ttf(&tables))
}

fn assemble_ttf(tables: &[(&[u8; 4], Vec<u8>)]) -> Vec<u8> {
    let nt = tables.len() as u16;
    let es = (nt as f32).log2().floor() as u16;
    let sr = (1u16 << es) * 16;
    let rs = nt * 16 - sr;

    let mut font = Vec::new();
    put_u32(&mut font, 0x00010000);
    put_u16(&mut font, nt);
    put_u16(&mut font, sr);
    put_u16(&mut font, es);
    put_u16(&mut font, rs);

    let data_start = 12 + nt as usize * 16;
    let mut data = Vec::new();
    let mut entries: Vec<(&[u8; 4], u32, u32, u32)> = Vec::new();

    for (tag, tbl) in tables {
        let off = (data_start + data.len()) as u32;
        entries.push((tag, off, tbl.len() as u32, checksum(tbl)));
        data.extend_from_slice(tbl);
        pad4(&mut data);
    }

    for (tag, off, len, cs) in &entries {
        font.extend_from_slice(*tag);
        put_u32(&mut font, *cs);
        put_u32(&mut font, *off);
        put_u32(&mut font, *len);
    }
    font.extend_from_slice(&data);

    // Fix head.checksumAdjust
    let nt2 = u16::from_be_bytes([font[4], font[5]]) as usize;
    for i in 0..nt2 {
        let d = 12 + i * 16;
        if &font[d..d + 4] == b"head" {
            let hoff = u32::from_be_bytes(font[d + 8..d + 12].try_into().unwrap()) as usize;
            font[hoff + 8] = 0; font[hoff + 9] = 0;
            font[hoff + 10] = 0; font[hoff + 11] = 0;
            let whole = checksum(&font);
            let adj = 0xB1B0AFBAu32.wrapping_sub(whole);
            font[hoff + 8] = (adj >> 24) as u8;
            font[hoff + 9] = (adj >> 16) as u8;
            font[hoff + 10] = (adj >> 8) as u8;
            font[hoff + 11] = adj as u8;
            break;
        }
    }
    font
}

// ---------------------------------------------------------------------------
// WOFF / EOT conversion
// ---------------------------------------------------------------------------

pub fn ttf_to_woff(ttf: &[u8]) -> Result<Vec<u8>, AppError> {
    if ttf.len() < 12 {
        return Err(AppError::ExportError("Invalid TTF".into()));
    }
    let nt = u16::from_be_bytes([ttf[4], ttf[5]]) as usize;

    let mut ttf_tables: Vec<([u8; 4], u32, Vec<u8>)> = Vec::new();
    for i in 0..nt {
        let d = 12 + i * 16;
        let mut tag = [0u8; 4];
        tag.copy_from_slice(&ttf[d..d + 4]);
        let _cs = u32::from_be_bytes(ttf[d + 4..d + 8].try_into().unwrap());
        let off = u32::from_be_bytes(ttf[d + 8..d + 12].try_into().unwrap()) as usize;
        let len = u32::from_be_bytes(ttf[d + 12..d + 16].try_into().unwrap()) as usize;
        let data = ttf[off..off + len].to_vec();
        ttf_tables.push((tag, checksum(&data), data));
    }

    let mut compressed: Vec<([u8; 4], Vec<u8>, u32, u32)> = Vec::new();
    for (tag, cs, data) in &ttf_tables {
        let enc = zlib_compress(data);
        let final_d = if enc.len() < data.len() { enc } else { data.clone() };
        compressed.push((*tag, final_d, data.len() as u32, *cs));
    }

    let hdr_sz = 44;
    let dir_sz = nt * 20;
    let mut data_buf = Vec::new();
    let mut dir_entries = Vec::new();
    for (tag, comp, orig_len, orig_cs) in &compressed {
        let off = (hdr_sz + dir_sz + data_buf.len()) as u32;
        dir_entries.push((*tag, off, comp.len() as u32, *orig_len, *orig_cs));
        data_buf.extend_from_slice(comp);
    }

    let total = hdr_sz + dir_sz + data_buf.len();
    let mut woff = Vec::with_capacity(total);
    put_u32(&mut woff, 0x774F4646); // 'wOFF'
    put_u32(&mut woff, 0x00010000); // flavor
    put_u32(&mut woff, total as u32);
    put_u16(&mut woff, nt as u16);
    put_u16(&mut woff, 0);
    put_u32(&mut woff, ttf.len() as u32); // totalSfntSize
    put_u16(&mut woff, 1); put_u16(&mut woff, 0);
    put_u32(&mut woff, 0); put_u32(&mut woff, 0); put_u32(&mut woff, 0);
    put_u32(&mut woff, 0); put_u32(&mut woff, 0);

    for (tag, off, comp_len, orig_len, orig_cs) in &dir_entries {
        woff.extend_from_slice(tag);
        put_u32(&mut woff, *off);
        put_u32(&mut woff, *comp_len);
        put_u32(&mut woff, *orig_len);
        put_u32(&mut woff, *orig_cs);
    }
    woff.extend_from_slice(&data_buf);
    Ok(woff)
}

fn zlib_compress(data: &[u8]) -> Vec<u8> {
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
    enc.write_all(data).unwrap();
    enc.finish().unwrap()
}

pub fn ttf_to_eot(ttf: &[u8]) -> Vec<u8> {
    let mut eot = Vec::with_capacity(70 + ttf.len());
    put_u32(&mut eot, (70 + ttf.len()) as u32); // eotSize
    put_u32(&mut eot, ttf.len() as u32);         // fontDataSize
    put_u32(&mut eot, 0x00010000);                // version
    put_u32(&mut eot, 0);                         // flags
    eot.extend_from_slice(&[0u8; 10]);            // panose
    put_u8(&mut eot, 1);                          // charset
    put_u8(&mut eot, 0);                          // italic
    put_u32(&mut eot, 400);                       // weight
    put_u16(&mut eot, 0);                         // fsType
    eot.extend_from_slice(&[0u8; 16]);            // unicodeRange
    eot.extend_from_slice(&[0u8; 8]);             // codePageRange
    put_u32(&mut eot, 0);                         // checkSumAdjustment
    eot.extend_from_slice(&[0u8; 8]);             // reserved
    eot.extend_from_slice(ttf);
    eot
}

// ---------------------------------------------------------------------------
// CSS / HTML generation
// ---------------------------------------------------------------------------

pub fn generate_css(glyphs: &[(String, char)], font_name: &str) -> String {
    let prefix = font_name.to_lowercase().replace(' ', "-");
    let mut s = String::new();

    s.push_str("@font-face {\n");
    s.push_str(&format!("  font-family: '{}';\n", font_name));
    s.push_str(&format!("  src: url('{}.eot');\n", font_name));
    s.push_str(&format!("  src: url('{}.eot?#iefix') format('embedded-opentype'),\n", font_name));
    s.push_str(&format!("       url('{}.woff2') format('woff2'),\n", font_name));
    s.push_str(&format!("       url('{}.woff') format('woff'),\n", font_name));
    s.push_str(&format!("       url('{}.ttf') format('truetype');\n", font_name));
    s.push_str("  font-weight: normal;\n");
    s.push_str("  font-style: normal;\n");
    s.push_str("  font-display: block;\n");
    s.push_str("}\n\n");

    s.push_str(&format!(".{}-icon {{\n", prefix));
    s.push_str(&format!("  font-family: '{}' !important;\n", font_name));
    s.push_str("  speak: never;\n  font-style: normal;\n  font-weight: normal;\n");
    s.push_str("  font-variant: normal;\n  text-transform: none;\n  line-height: 1;\n");
    s.push_str("  -webkit-font-smoothing: antialiased;\n  -moz-osx-font-smoothing: grayscale;\n");
    s.push_str("}\n\n");

    for (name, uc) in glyphs {
        let cn = name.to_lowercase().replace(' ', "-");
        s.push_str(&format!(".{}-{}::before {{ content: '\\{:04X}'; }}\n", prefix, cn, *uc as u32));
    }
    s
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn generate_html_demo(glyphs: &[(String, char)], font_name: &str) -> String {
    let prefix = font_name.to_lowercase().replace(' ', "-");
    let mut h = String::new();
    h.push_str("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"UTF-8\">\n");
    h.push_str(&format!("<title>{} Icon Font Demo</title>\n", font_name));
    h.push_str(&format!("<link rel=\"stylesheet\" href=\"{}.css\">\n", font_name));
    h.push_str("<style>\n");
    h.push_str("body{font-family:-apple-system,sans-serif;padding:20px;max-width:960px;margin:0 auto}\n");
    h.push_str("h1{font-size:24px;margin-bottom:20px}\n");
    h.push_str(".search{width:100%;padding:8px 12px;font-size:14px;border:1px solid #ccc;border-radius:4px;margin-bottom:20px;box-sizing:border-box}\n");
    h.push_str(".grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(120px,1fr));gap:12px}\n");
    h.push_str(".card{border:1px solid #eee;border-radius:8px;padding:16px;text-align:center;cursor:pointer;transition:all .2s}\n");
    h.push_str(".card:hover{border-color:#4A90D9;box-shadow:0 2px 8px rgba(0,0,0,.1)}\n");
    h.push_str(&format!(".preview{{font-family:'{}';font-size:32px;margin-bottom:8px}}\n", font_name));
    h.push_str(".name{font-size:11px;color:#666;word-break:break-all}\n");
    h.push_str(".uc{font-size:10px;color:#999;font-family:monospace}\n");
    h.push_str(".toast{position:fixed;top:20px;right:20px;background:#4CAF50;color:#fff;padding:8px 16px;border-radius:4px;display:none}\n");
    h.push_str("</style></head><body>\n");
    h.push_str(&format!("<h1>{}</h1>\n", font_name));
    h.push_str("<input class=\"search\" type=\"text\" placeholder=\"Search icons...\" oninput=\"filter(this.value)\">\n");
    h.push_str("<div class=\"grid\" id=\"g\">\n");
    for (name, uc) in glyphs {
        let cn = name.to_lowercase().replace(' ', "-");
        let hex = format!("{:04X}", *uc as u32);
        let safe_name = html_escape(name);
        let safe_cn = html_escape(&cn);
        let safe_lower = html_escape(&name.to_lowercase());
        h.push_str(&format!("<div class=\"card\" data-n=\"{}\" onclick=\"cp('{}-{}')\"><div class=\"preview\">&#x{};</div><div class=\"name\">{}</div><div class=\"uc\">0x{}</div></div>\n",
            safe_lower, prefix, safe_cn, hex, safe_name, hex));
    }
    h.push_str("</div><div class=\"toast\" id=\"t\">Copied!</div>\n");
    h.push_str("<script>\nfunction cp(c){navigator.clipboard.writeText(c);var e=document.getElementById('t');e.style.display='block';setTimeout(function(){e.style.display='none'},1500)}\n");
    h.push_str("function filter(q){q=q.toLowerCase();document.querySelectorAll('.card').forEach(function(c){c.style.display=c.dataset.n.includes(q)?'':'none'})}\n");
    h.push_str("</script></body></html>\n");
    h
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

pub fn generate_font(glyphs: &[GlyphEntry], options: &FontExportOptions) -> Result<FontExportResult, AppError> {
    let em = 1000u16;

    let mut gdata: Vec<(char, GlyphData)> = Vec::new();
    for e in glyphs {
        let d = if e.svg_path_data.trim().is_empty() {
            empty_glyph(em)
        } else {
            svg_to_glyph(&e.svg_path_data, em)?
        };
        gdata.push((e.unicode, d));
    }

    let ttf = build_ttf(&gdata, &options.font_name, em)?;
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();

    for fmt in &options.formats {
        match fmt {
            FontFormat::Ttf => {
                files.push((format!("{}.ttf", options.font_name), ttf.clone()));
            }
            FontFormat::Woff => {
                let w = ttf_to_woff(&ttf)?;
                files.push((format!("{}.woff", options.font_name), w));
            }
            FontFormat::Woff2 => {
                return Err(AppError::ExportError("WOFF2 not yet supported — use TTF or WOFF".into()));
            }
            FontFormat::Eot => {
                files.push((format!("{}.eot", options.font_name), ttf_to_eot(&ttf)));
            }
        }
    }

    if options.include_css {
        let gi: Vec<(String, char)> = glyphs.iter().map(|g| (g.icon_name.clone(), g.unicode)).collect();
        files.push((format!("{}.css", options.font_name), generate_css(&gi, &options.font_name).into_bytes()));
    }
    if options.include_demo {
        let gi: Vec<(String, char)> = glyphs.iter().map(|g| (g.icon_name.clone(), g.unicode)).collect();
        files.push((format!("{}.html", options.font_name), generate_html_demo(&gi, &options.font_name).into_bytes()));
    }

    Ok(FontExportResult { files })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn rect_svg() -> String {
        "<?xml version=\"1.0\"?><svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" width=\"24\" height=\"24\"><rect x=\"4\" y=\"4\" width=\"16\" height=\"16\"/></svg>".into()
    }
    fn circle_svg() -> String {
        "<?xml version=\"1.0\"?><svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" width=\"24\" height=\"24\"><circle cx=\"12\" cy=\"12\" r=\"10\"/></svg>".into()
    }
    fn cubic_svg() -> String {
        "<?xml version=\"1.0\"?><svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" width=\"24\" height=\"24\"><path d=\"M4 12 C4 6, 20 6, 20 12 C20 18, 4 18, 4 12 Z\"/></svg>".into()
    }

    #[test]
    fn test_svg_to_glyph_simple_rect() {
        let g = svg_to_glyph(&rect_svg(), 1000).unwrap();
        assert!(!g.contours.is_empty());
        let pts: usize = g.contours.iter().map(|c| c.len()).sum();
        assert!(pts >= 4, "rect needs >= 4 points, got {}", pts);
    }

    #[test]
    fn test_svg_to_glyph_circle() {
        let g = svg_to_glyph(&circle_svg(), 1000).unwrap();
        assert!(!g.contours.is_empty());
        let pts: usize = g.contours.iter().map(|c| c.len()).sum();
        assert!(pts >= 8, "circle approx needs >= 8 points, got {}", pts);
    }

    #[test]
    fn test_svg_to_glyph_cubic_to_quadratic() {
        let g = svg_to_glyph(&cubic_svg(), 1000).unwrap();
        assert!(!g.contours.is_empty());
        let pts: usize = g.contours.iter().map(|c| c.len()).sum();
        assert!(pts >= 8, "cubic→quad needs >= 8 points, got {}", pts);
    }

    #[test]
    fn test_generate_ttf_not_empty() {
        let entries = vec![('\u{E001}', svg_to_glyph(&rect_svg(), 1000).unwrap())];
        let ttf = build_ttf(&entries, "TestFont", 1000).unwrap();
        assert!(!ttf.is_empty());
        assert!(ttf.len() > 100, "TTF too small: {}", ttf.len());
    }

    #[test]
    fn test_generate_ttf_valid_header() {
        let entries = vec![('\u{E001}', svg_to_glyph(&rect_svg(), 1000).unwrap())];
        let ttf = build_ttf(&entries, "TestFont", 1000).unwrap();
        assert_eq!(&ttf[0..4], &[0x00, 0x01, 0x00, 0x00]);
        let nt = u16::from_be_bytes([ttf[4], ttf[5]]);
        assert_eq!(nt, 10);
    }

    #[test]
    fn test_ttf_to_woff_smaller() {
        let entries = vec![
            ('\u{E001}', svg_to_glyph(&rect_svg(), 1000).unwrap()),
            ('\u{E002}', svg_to_glyph(&circle_svg(), 1000).unwrap()),
        ];
        let ttf = build_ttf(&entries, "TestFont", 1000).unwrap();
        let woff = ttf_to_woff(&ttf).unwrap();
        assert!(woff.len() < ttf.len(), "WOFF {} >= TTF {}", woff.len(), ttf.len());
    }

    #[test]
    fn test_unicode_allocation() {
        let entries: Vec<(char, GlyphData)> = vec![
            ('\u{E000}', empty_glyph(1000)),
            ('\u{E001}', empty_glyph(1000)),
            ('\u{E002}', empty_glyph(1000)),
        ];
        let ttf = build_ttf(&entries, "Test", 1000).unwrap();
        assert!(!ttf.is_empty());
        assert_eq!(entries[0].0, '\u{E000}');
        assert_eq!(entries[1].0, '\u{E001}');
        assert_eq!(entries[2].0, '\u{E002}');
    }

    #[test]
    fn test_generate_css_contains_font_face() {
        let gl = vec![("home".into(), '\u{E001}'), ("search".into(), '\u{E002}')];
        let css = generate_css(&gl, "MyIcons");
        assert!(css.contains("@font-face"));
        assert!(css.contains("font-family: 'MyIcons'"));
        assert!(css.contains(".myicons-home::before"));
        assert!(css.contains("content: '\\E001'"));
    }

    #[test]
    fn test_generate_html_demo_contains_all_icons() {
        let gl = vec![("home".into(), '\u{E001}'), ("search".into(), '\u{E002}'), ("settings".into(), '\u{E003}')];
        let html = generate_html_demo(&gl, "MyIcons");
        assert!(html.contains("home"));
        assert!(html.contains("search"));
        assert!(html.contains("settings"));
    }

    #[test]
    fn test_empty_glyph_set() {
        let entries: Vec<(char, GlyphData)> = vec![];
        let ttf = build_ttf(&entries, "Empty", 1000).unwrap();
        assert!(!ttf.is_empty());
        let nt = u16::from_be_bytes([ttf[4], ttf[5]]);
        assert_eq!(nt, 10);
    }

    #[test]
    fn test_font_round_trip() {
        let entries = vec![('\u{E001}', svg_to_glyph(&rect_svg(), 1000).unwrap())];
        let ttf = build_ttf(&entries, "TestFont", 1000).unwrap();
        assert!(ttf.len() > 12);
        assert_eq!(&ttf[0..4], &[0x00, 0x01, 0x00, 0x00]);

        let nt = u16::from_be_bytes([ttf[4], ttf[5]]) as usize;
        let mut found_head = false;
        for i in 0..nt {
            let d = 12 + i * 16;
            if &ttf[d..d + 4] == b"head" {
                found_head = true;
                let off = u32::from_be_bytes(ttf[d + 8..d + 12].try_into().unwrap()) as usize;
                let len = u32::from_be_bytes(ttf[d + 12..d + 16].try_into().unwrap()) as usize;
                let head = &ttf[off..off + len];
                assert!(head.len() >= 12);
                assert_eq!(&head[12..16], &[0x5F, 0x0F, 0x3C, 0xF5]);
                break;
            }
        }
        assert!(found_head, "head table not found");
    }
}
