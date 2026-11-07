//! Packs the Hive Auto CS core module into a `packet-mod` component with its grants.

use std::{borrow::Cow, error::Error, path::Path};

use wasm_encoder::{ComponentNameSection, ComponentSection, CustomSection};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: hive-autocs-pack CORE.wasm COMPONENT.wasm".into());
    }
    let core = std::fs::read(&args[0])?;
    let mut component = wit_component::ComponentEncoder::default()
        .module(&core)?
        .validate(true)
        .encode()?;

    let mut name = ComponentNameSection::new();
    name.component("Hive Auto CS");
    name.append_to_component(&mut component);
    append_section(
        &mut component,
        "description",
        b"Type !autocs on The Hive: creates a BedWars Squads custom server, loads your preset, sets the map and puts your party on RED, everyone else on BLUE.",
    );
    append_section(&mut component, "cinnabar:grants", include_bytes!("../grants.json"));
    append_section(
        &mut component,
        "cinnabar:extension/icon@1",
        include_bytes!("../../../../assets/base-icon.png"),
    );
    wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::all())
        .validate_all(&component)?;
    check_interfaces(&component)?;

    let output = Path::new(&args[1]);
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output, &component)?;
    println!("Built Cinnabar packet mod: {} bytes", component.len());
    Ok(())
}

/// The WASI interfaces the component may import: standard output only, which the packet runtime
/// links with the WASI grant (it never links files or network). The other imports are type-only
/// interfaces and types from `mod-api`'s worlds, which wasmtime needs no implementation for.
const ALLOWED_WASI: [&str; 3] = ["wasi:io/error@0.2.0", "wasi:io/streams@0.2.0", "wasi:cli/stdout@0.2.0"];

/// Prints the component's top-level imports and exports and rejects unexpected imports.
fn check_interfaces(component: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut depth = 0usize;
    for payload in wasmparser::Parser::new(0).parse_all(component) {
        match payload? {
            wasmparser::Payload::ModuleSection { .. } | wasmparser::Payload::ComponentSection { .. } => {
                depth += 1
            }
            wasmparser::Payload::End(_) => depth = depth.saturating_sub(1),
            wasmparser::Payload::ComponentImportSection(imports) if depth == 0 => {
                for import in imports {
                    let name = import?.name.0;
                    println!("import {name}");
                    if name.starts_with("wasi:") && !ALLOWED_WASI.contains(&name) {
                        return Err(format!("unexpected import {name}").into());
                    }
                }
            }
            wasmparser::Payload::ComponentExportSection(exports) if depth == 0 => {
                for export in exports {
                    println!("export {}", export?.name.0);
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn append_section(component: &mut Vec<u8>, name: &str, data: &[u8]) {
    CustomSection { name: Cow::Borrowed(name), data: Cow::Borrowed(data) }.append_to_component(component);
}
