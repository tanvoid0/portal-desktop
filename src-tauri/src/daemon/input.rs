//! Mouse/keyboard injection for `/control`'s trackpad and keyboard frames —
//! the OS side of `control::handle_client_frame`. One [`State`] per socket:
//! it owns the `Enigo` handle and the set of virtual keys this phone is
//! currently holding down, so a phone that drops mid-`Ctrl` doesn't leave
//! this PC stuck holding it ([`State::release_held`] runs on disconnect).
//!
//! Wire shapes mirror `portal_remote`'s C# server (`Input/InputActions.cs`,
//! `Input/WinInput.cs`) field-for-field — this is the same Android app
//! (`net/Protocol.kt`) pointed at a new daemon, not a new protocol, so the
//! shapes were read off that source rather than sniffed from the wire.
//! `cast`/`power`/`media`/`touch`/`player` and friends are out of scope here
//! (screen mirroring and casting are later phases); only the trackpad and
//! keyboard surface is implemented.

use std::collections::HashSet;

use enigo::{Axis, Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use serde_json::Value;

/// Message types this module handles. `control::handle_client_frame` gates
/// exactly these on the `remote.input` capability; everything else (`ping`,
/// the broadcast fanout) is unaffected.
pub const FRAME_TYPES: &[&str] = &[
    "mouse_move",
    "mouse_move_abs",
    "mouse_click",
    "scroll",
    "key",
    "tap",
    "combo",
    "text",
];

/// One `/control` socket's input state.
pub struct State {
    enigo: Enigo,
    /// Virtual keys sent down and not yet up. See the module doc.
    held: HashSet<u16>,
}

impl State {
    /// `None` when the platform input backend could not be opened — no
    /// display server, or (per `enigo`'s own docs) missing OS permission.
    /// `/control` still answers `ping` and forwards events either way; it
    /// just can't move the mouse, and says so if a phone asks it to.
    pub fn new() -> Option<Self> {
        match Enigo::new(&Settings::default()) {
            Ok(enigo) => Some(Self { enigo, held: HashSet::new() }),
            Err(e) => {
                eprintln!("[daemon] input injection unavailable: {e}");
                None
            }
        }
    }

    /// Key-up everything still held. The control socket's close path, same
    /// reasoning as `InputActions.ReleaseHeld` on the C# server.
    pub fn release_held(&mut self) {
        for vk in std::mem::take(&mut self.held) {
            let _ = self.enigo.key(Key::Other(u32::from(vk)), Direction::Release);
        }
    }

    /// Apply one message. The caller (`control::handle_client_frame`) has
    /// already matched `t` against [`FRAME_TYPES`] and checked the
    /// capability; this is the part that can still fail (an unknown key
    /// name, a malformed payload, the OS call itself).
    pub fn dispatch(&mut self, t: &str, msg: &Value) -> Result<(), String> {
        match t {
            "mouse_move" => {
                let dx = get_i32(msg, "dx").unwrap_or(0);
                let dy = get_i32(msg, "dy").unwrap_or(0);
                if dx != 0 || dy != 0 {
                    self.enigo
                        .move_mouse(dx, dy, Coordinate::Rel)
                        .map_err(|e| e.to_string())?;
                }
                Ok(())
            }
            "mouse_move_abs" => self.mouse_move_abs(msg),
            "mouse_click" => self.mouse_click(msg),
            "scroll" => self.scroll(msg),
            "key" => self.key(msg),
            "tap" => {
                let vk = resolve_vk(msg)?;
                self.enigo
                    .key(Key::Other(u32::from(vk)), Direction::Click)
                    .map_err(|e| e.to_string())
            }
            "combo" => self.combo(msg),
            "text" => {
                let s = msg.get("s").and_then(Value::as_str).unwrap_or("");
                self.enigo.text(s).map_err(|e| e.to_string())
            }
            other => Err(format!("unknown input message type: {other}")),
        }
    }

    /// `nx`/`ny` (0..1 of the primary display) take precedence over pixel
    /// `x`/`y`, same as `InputActions.cs`. Unlike the C# server this has no
    /// `mon`/`win` — picking a non-primary monitor or a mirrored window is
    /// screen-mirroring territory, not trackpad input, so it's left out
    /// rather than silently ignored on the server that actually has it.
    fn mouse_move_abs(&mut self, msg: &Value) -> Result<(), String> {
        if let Some(nx) = get_f64(msg, "nx") {
            let ny = get_f64(msg, "ny")
                .ok_or_else(|| "mouse_move_abs with 'nx' also needs 'ny'".to_string())?;
            let (w, h) = self.enigo.main_display().map_err(|e| e.to_string())?;
            let x = (nx.clamp(0.0, 1.0) * f64::from(w - 1)).round() as i32;
            let y = (ny.clamp(0.0, 1.0) * f64::from(h - 1)).round() as i32;
            return self
                .enigo
                .move_mouse(x, y, Coordinate::Abs)
                .map_err(|e| e.to_string());
        }
        let x = get_i32(msg, "x").ok_or_else(|| "mouse_move_abs needs 'x'".to_string())?;
        let y = get_i32(msg, "y").ok_or_else(|| "mouse_move_abs needs 'y'".to_string())?;
        self.enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| e.to_string())
    }

    fn mouse_click(&mut self, msg: &Value) -> Result<(), String> {
        let btn = match msg.get("btn").and_then(Value::as_str).unwrap_or("left") {
            "left" => Button::Left,
            "right" => Button::Right,
            "middle" => Button::Middle,
            "back" => Button::Back,
            "forward" => Button::Forward,
            other => return Err(format!("unknown mouse button: {other}")),
        };
        // No explicit 'down' means a full press-and-release, same as the C# server.
        let direction = match msg.get("down").and_then(Value::as_bool) {
            Some(true) => Direction::Press,
            Some(false) => Direction::Release,
            None => Direction::Click,
        };
        self.enigo.button(btn, direction).map_err(|e| e.to_string())
    }

    /// `dy`/`dx` are raw wheel deltas, 120 == one notch, positive scrolls
    /// up/right (the convention `Protocol.kt`/`InputActions.cs` document).
    /// `enigo::Mouse::scroll` counts notches and treats a positive vertical
    /// length as *down*, so the vertical axis is negated going in; the
    /// horizontal sign already matches.
    // ponytail: rounds to the nearest notch per call rather than carrying a
    // fractional remainder, so a sub-notch delta (<60) is dropped instead of
    // accumulating. Add a remainder if a real trackpad ever feels sticky.
    fn scroll(&mut self, msg: &Value) -> Result<(), String> {
        let dy = get_i32(msg, "dy").unwrap_or(0);
        let dx = get_i32(msg, "dx").unwrap_or(0);
        if let Some(notches) = notches(-dy) {
            self.enigo
                .scroll(notches, Axis::Vertical)
                .map_err(|e| e.to_string())?;
        }
        if let Some(notches) = notches(dx) {
            self.enigo
                .scroll(notches, Axis::Horizontal)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn key(&mut self, msg: &Value) -> Result<(), String> {
        let vk = resolve_vk(msg)?;
        // No explicit 'down' defaults to held-down, same as the C# server.
        let down = msg.get("down").and_then(Value::as_bool).unwrap_or(true);
        let direction = if down { Direction::Press } else { Direction::Release };
        self.enigo
            .key(Key::Other(u32::from(vk)), direction)
            .map_err(|e| e.to_string())?;
        if down {
            self.held.insert(vk);
        } else {
            self.held.remove(&vk);
        }
        Ok(())
    }

    /// Press in order, release in reverse, so modifiers wrap the final key —
    /// same as `InputActions.cs`'s `combo`.
    fn combo(&mut self, msg: &Value) -> Result<(), String> {
        let keys = read_combo(msg)?;
        if keys.is_empty() {
            return Err("combo needs a non-empty 'keys' list".to_string());
        }
        for &vk in &keys {
            self.enigo
                .key(Key::Other(u32::from(vk)), Direction::Press)
                .map_err(|e| e.to_string())?;
        }
        for &vk in keys.iter().rev() {
            self.enigo
                .key(Key::Other(u32::from(vk)), Direction::Release)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

/// A raw wheel delta as whole notches, or `None` for "nothing to send" —
/// either the delta was zero or it rounded away to nothing.
fn notches(delta: i32) -> Option<i32> {
    let n = (f64::from(delta) / 120.0).round() as i32;
    (n != 0).then_some(n)
}

/// Resolve a key from either a numeric `vk` (a raw Windows virtual-key code,
/// 0..0xFF) or a named `key` — the same two ways `InputActions.cs`'s
/// `ResolveVk` accepts.
fn resolve_vk(msg: &Value) -> Result<u16, String> {
    if let Some(raw) = msg.get("vk").and_then(Value::as_i64) {
        return u16::try_from(raw)
            .ok()
            .filter(|&vk| vk <= 0xFF)
            .ok_or_else(|| format!("vk out of range: {raw}"));
    }
    let name = msg
        .get("key")
        .and_then(Value::as_str)
        .ok_or_else(|| "unknown key name: <missing>".to_string())?;
    named_vk(name).ok_or_else(|| format!("unknown key name: {name}"))
}

fn read_combo(msg: &Value) -> Result<Vec<u16>, String> {
    let Some(arr) = msg.get("keys").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    arr.iter()
        .map(|item| {
            if let Some(n) = item.as_i64() {
                u16::try_from(n)
                    .ok()
                    .filter(|&vk| vk <= 0xFF)
                    .ok_or_else(|| format!("vk out of range: {n}"))
            } else if let Some(name) = item.as_str() {
                named_vk(name).ok_or_else(|| format!("unknown key name: {name}"))
            } else {
                Err("combo keys must be numbers or names".to_string())
            }
        })
        .collect()
}

/// Virtual-key codes for the keys the phone UI sends by name. Same table as
/// `InputActions.cs`'s `BuildVkTable`, since both ends have to agree on what
/// e.g. `"home"` means.
fn named_vk(name: &str) -> Option<u16> {
    let lower = name.to_ascii_lowercase();
    let named = match lower.as_str() {
        "backspace" => 0x08,
        "tab" => 0x09,
        "enter" => 0x0D,
        "shift" => 0x10,
        "ctrl" => 0x11,
        "alt" => 0x12,
        "pause" => 0x13,
        "capslock" => 0x14,
        "esc" => 0x1B,
        "space" => 0x20,
        "pgup" => 0x21,
        "pgdn" => 0x22,
        "end" => 0x23,
        "home" => 0x24,
        "left" => 0x25,
        "up" => 0x26,
        "right" => 0x27,
        "down" => 0x28,
        "printscreen" => 0x2C,
        "insert" => 0x2D,
        "delete" => 0x2E,
        "win" => 0x5B,
        "apps" => 0x5D,
        "numlock" => 0x90,
        "scrolllock" => 0x91,
        "lshift" => 0xA0,
        "rshift" => 0xA1,
        "lctrl" => 0xA2,
        "rctrl" => 0xA3,
        "lalt" => 0xA4,
        "ralt" => 0xA5,
        "browser_back" => 0xA6,
        "browser_forward" => 0xA7,
        "mute" => 0xAD,
        "vol_down" => 0xAE,
        "vol_up" => 0xAF,
        "next" => 0xB0,
        "prev" => 0xB1,
        "stop" => 0xB2,
        "play_pause" => 0xB3,
        _ => return named_vk_fallback(&lower),
    };
    Some(named)
}

/// F1..F24, and the single a..z / 0..9 keys — arithmetic rather than another
/// 36-entry match, same as the C# table builds them with a loop.
fn named_vk_fallback(lower: &str) -> Option<u16> {
    if let Some(n) = lower.strip_prefix('f') {
        let i: u16 = n.parse().ok()?;
        if (1..=24).contains(&i) {
            return Some(0x6F + i);
        }
    }
    if lower.len() == 1 {
        let c = lower.as_bytes()[0];
        if c.is_ascii_lowercase() {
            return Some(u16::from(c - b'a') + 0x41); // VK_A..VK_Z
        }
        if c.is_ascii_digit() {
            return Some(u16::from(c)); // VK_0..VK_9 == ASCII '0'..'9'
        }
    }
    None
}

fn get_i32(msg: &Value, key: &str) -> Option<i32> {
    msg.get(key).and_then(Value::as_i64).and_then(|v| i32::try_from(v).ok())
}

fn get_f64(msg: &Value, key: &str) -> Option<f64> {
    msg.get(key).and_then(Value::as_f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn named_keys_match_the_android_clients_table() {
        assert_eq!(named_vk("home"), Some(0x24));
        assert_eq!(named_vk("ENTER"), Some(0x0D)); // case-insensitive
        assert_eq!(named_vk("a"), Some(0x41));
        assert_eq!(named_vk("z"), Some(0x5A));
        assert_eq!(named_vk("5"), Some(b'5' as u16));
        assert_eq!(named_vk("f1"), Some(0x70));
        assert_eq!(named_vk("f24"), Some(0x87));
        assert_eq!(named_vk("f25"), None);
        assert_eq!(named_vk("nonsense"), None);
    }

    #[test]
    fn resolve_vk_prefers_numeric_vk_over_key_name() {
        assert_eq!(resolve_vk(&json!({ "vk": 65, "key": "home" })), Ok(65));
        assert_eq!(resolve_vk(&json!({ "key": "enter" })), Ok(0x0D));
        assert!(resolve_vk(&json!({ "vk": 999 })).is_err());
        assert!(resolve_vk(&json!({ "key": "nonsense" })).is_err());
        assert!(resolve_vk(&json!({})).is_err());
    }

    #[test]
    fn combo_resolves_mixed_names_and_vk_numbers() {
        assert_eq!(
            read_combo(&json!({ "keys": ["ctrl", 0x41] })),
            Ok(vec![0x11, 0x41])
        );
        assert_eq!(read_combo(&json!({})), Ok(Vec::new()));
        assert!(read_combo(&json!({ "keys": [true] })).is_err());
    }

    #[test]
    fn scroll_notches_round_to_the_nearest_click_and_drop_sub_notch_deltas() {
        assert_eq!(notches(120), Some(1));
        assert_eq!(notches(-240), Some(-2));
        assert_eq!(notches(50), None); // rounds to 0 — dropped, not accumulated
        assert_eq!(notches(0), None);
    }
}
