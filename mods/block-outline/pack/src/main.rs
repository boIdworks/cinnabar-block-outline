use std::{borrow::Cow, error::Error, path::Path};

use wasm_encoder::{ComponentNameSection, ComponentSection, CustomSection};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: block-outline-pack CORE.wasm COMPONENT.wasm".into());
    }

    let core = std::fs::read(&args[0])?;
    let mut component = wit_component::ComponentEncoder::default()
        .module(&core)?
        .validate(true)
        .encode()?;

    let mut name = ComponentNameSection::new();
    name.component("Block Outline");
    name.append_to_component(&mut component);
    append_section(
        &mut component,
        "description",
        b"Outlines the block you are looking at. Press F8 to change its colour, opacity and thickness.",
    );
    append_section(
        &mut component,
        "cinnabar:grants",
        include_bytes!("../grants.json"),
    );
    // The F8 menu, loaded by the host without spending guest fuel.
    append_section(
        &mut component,
        "cinnabar:extension/panel-bundle@1",
        block_outline::panel::bundle().as_bytes(),
    );
    append_section(
        &mut component,
        "cinnabar:extension/icon@1",
        include_bytes!("../../../../assets/base-icon.png"),
    );

    wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::all())
        .validate_all(&component)?;
    for payload in wasmparser::Parser::new(0).parse_all(&component) {
        match payload? {
            wasmparser::Payload::ImportSection(imports) => {
                for import in imports.into_imports() {
                    if import?.module.starts_with("wasi") {
                        return Err("Cinnabar mods must not import WASI".into());
                    }
                }
            }
            wasmparser::Payload::ComponentImportSection(imports) => {
                for import in imports {
                    if import?.name.0.starts_with("wasi") {
                        return Err("Cinnabar mods must not import WASI".into());
                    }
                }
            }
            _ => {}
        }
    }

    let output = Path::new(&args[1]);
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output, &component)?;
    println!("Built Cinnabar component: {} bytes", component.len());
    Ok(())
}

fn append_section(component: &mut Vec<u8>, name: &str, data: &[u8]) {
    CustomSection {
        name: Cow::Borrowed(name),
        data: Cow::Borrowed(data),
    }
    .append_to_component(component);
}
