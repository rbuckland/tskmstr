//! `tskmstr-tray render-icon --out-dir DIR`: writes the application icon in
//! the formats the packaging needs (PNG set, macOS `.icns`, Windows `.ico`)
//! plus tray-icon previews. Used by the release pipeline so no image tooling
//! is required on the build machines.

use std::path::Path;

use anyhow::{Context, Result};

use crate::icon::{render_pixmap, Layout};

pub fn run(out_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(out_dir)?;

    let sizes = [16u32, 32, 48, 64, 128, 256, 512, 1024];
    let mut pngs = Vec::new();
    for size in sizes {
        let png = render_pixmap(0, Layout::AppIcon, size)
            .encode_png()
            .context("encoding app icon")?;
        std::fs::write(out_dir.join(format!("app-icon-{size}.png")), &png)?;
        pngs.push((size, png));
    }
    std::fs::write(out_dir.join("tskmstr.icns"), icns(&pngs))?;
    std::fs::write(
        out_dir.join("tskmstr.ico"),
        ico(&pngs, &[16, 32, 48, 64, 128, 256]),
    )?;

    for (name, layout) in [("menubar", Layout::MenuBar), ("square", Layout::Square)] {
        for n in [0usize, 7, 43] {
            let png = render_pixmap(n, layout, 64).encode_png()?;
            std::fs::write(out_dir.join(format!("tray-{name}-{n}.png")), png)?;
        }
    }

    println!("icons written to {}", out_dir.display());
    Ok(())
}

/// Apple Icon Image: a TOC-less container of PNG entries, one per size.
fn icns(pngs: &[(u32, Vec<u8>)]) -> Vec<u8> {
    // OSType per pixel size (PNG payloads are accepted for all of these).
    let types: &[(u32, &[u8; 4])] = &[
        (16, b"icp4"),
        (32, b"icp5"),
        (64, b"icp6"),
        (128, b"ic07"),
        (256, b"ic08"),
        (512, b"ic09"),
        (1024, b"ic10"),
    ];
    let mut body = Vec::new();
    for (size, ty) in types {
        if let Some((_, png)) = pngs.iter().find(|(s, _)| s == size) {
            body.extend_from_slice(*ty);
            body.extend_from_slice(&((png.len() + 8) as u32).to_be_bytes());
            body.extend_from_slice(png);
        }
    }
    let mut out = Vec::with_capacity(body.len() + 8);
    out.extend_from_slice(b"icns");
    out.extend_from_slice(&((body.len() + 8) as u32).to_be_bytes());
    out.extend_from_slice(&body);
    out
}

/// Windows ICO with PNG-compressed images (supported since Windows Vista).
fn ico(pngs: &[(u32, Vec<u8>)], sizes: &[u32]) -> Vec<u8> {
    let entries: Vec<&(u32, Vec<u8>)> = sizes
        .iter()
        .filter_map(|s| pngs.iter().find(|(ps, _)| ps == s))
        .collect();
    let mut out = Vec::new();
    out.extend_from_slice(&0u16.to_le_bytes()); // reserved
    out.extend_from_slice(&1u16.to_le_bytes()); // type: icon
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * entries.len() as u32;
    for (size, png) in &entries {
        let dim = |s: u32| if s >= 256 { 0u8 } else { s as u8 };
        out.push(dim(*size)); // width  (0 means 256)
        out.push(dim(*size)); // height
        out.push(0); // palette
        out.push(0); // reserved
        out.extend_from_slice(&1u16.to_le_bytes()); // planes
        out.extend_from_slice(&32u16.to_le_bytes()); // bpp
        out.extend_from_slice(&(png.len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += png.len() as u32;
    }
    for (_, png) in &entries {
        out.extend_from_slice(png);
    }
    out
}
