//! Cinnabar's own HUD and panel admission, compiled to wasm so a script can feed it the mod's
//! documents and patches exactly as the host would receive them.
use std::cell::RefCell;

use ui::mod_panel::{HudIndex, Panel, PanelBundle, PanelPatch, Surface, SurfacePatch};

#[derive(Default)]
struct State {
    hud: Option<(Surface, HudIndex)>,
    bundle: Option<PanelBundle>,
    panel: Option<Panel>,
    error: String,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut buffer = vec![0u8; len];
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    ptr
}

fn text(ptr: *mut u8, len: usize) -> String {
    // SAFETY: `ptr` came from `alloc(len)` and the caller filled it with UTF-8.
    let bytes = unsafe { Vec::from_raw_parts(ptr, len, len) };
    String::from_utf8(bytes).unwrap_or_default()
}

fn finish(result: Result<(), String>) -> i32 {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        match result {
            Ok(()) => {
                state.error.clear();
                0
            }
            Err(error) => {
                state.error = error;
                1
            }
        }
    })
}

/// `hud.set-surface`.
#[unsafe(no_mangle)]
pub extern "C" fn hud_set(ptr: *mut u8, len: usize) -> i32 {
    let json = text(ptr, len);
    let result = serde_json::from_str::<Surface>(&json)
        .map_err(|error| format!("invalid HUD surface: {error}"))
        .and_then(|surface| HudIndex::new(&surface).map(|index| (surface, index)))
        .map(|pair| STATE.with(|state| state.borrow_mut().hud = Some(pair)));
    finish(result)
}

/// `hud.update-surface` on the last admitted surface.
#[unsafe(no_mangle)]
pub extern "C" fn hud_patch(ptr: *mut u8, len: usize) -> i32 {
    let json = text(ptr, len);
    let result = STATE.with(|state| {
        let mut state = state.borrow_mut();
        let Some((surface, index)) = &state.hud else {
            return Err("no HUD surface to update".to_owned());
        };
        let patched = SurfacePatch::parse(&json)?.apply_hud(surface, index)?;
        state.hud = Some(patched);
        Ok(())
    });
    finish(result)
}

/// The panel bundle the component embeds.
#[unsafe(no_mangle)]
pub extern "C" fn panel_set(ptr: *mut u8, len: usize) -> i32 {
    let json = text(ptr, len);
    let result = PanelBundle::parse(&json).map(|bundle| {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.panel = Some(bundle.panel.clone());
            state.bundle = Some(bundle);
        })
    });
    finish(result)
}

/// `panel.update-content` on the current panel.
#[unsafe(no_mangle)]
pub extern "C" fn panel_patch(ptr: *mut u8, len: usize) -> i32 {
    let json = text(ptr, len);
    let result = STATE.with(|state| {
        let mut state = state.borrow_mut();
        let (Some(bundle), Some(panel)) = (&state.bundle, &state.panel) else {
            return Err("no panel".to_owned());
        };
        let next = PanelPatch::parse(&json)?.apply_bundle(panel, bundle)?;
        state.panel = Some(next);
        Ok(())
    });
    finish(result)
}

#[unsafe(no_mangle)]
pub extern "C" fn error_ptr() -> *const u8 {
    STATE.with(|state| state.borrow().error.as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn error_len() -> usize {
    STATE.with(|state| state.borrow().error.len())
}
