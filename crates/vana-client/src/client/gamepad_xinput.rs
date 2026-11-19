//! XInput as gamepad.rs's fallback on Windows, used while gilrs (Windows.Gaming.Input) has no pad. Some Wine front
//! ends on Android hook XInputGetState in the process but never make a WGI device, so gilrs sees nothing.
//! Retail's own XInput branch: FUN_10008e10 loads xinputdll.dll from the game folder (which links XINPUT1_3.dll) and
//! takes XInputGetState_ / XInputSetState_ / XInputEnable_; FUN_10005a40 reads one slot a frame through FUN_10008ea0
//! and turns it into the pad word with table 0x1032a818. We load the system DLLs ourselves, xinput1_3 first as
//! retail's, and keep the values raw as retail reads them (no dead zone or rescale here).

use super::gamepad::Pad;

/// XINPUT_GAMEPAD.
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct XGamepad {
    pub buttons: u16,
    pub left_trigger: u8,
    pub right_trigger: u8,
    pub thumb_lx: i16,
    pub thumb_ly: i16,
    pub thumb_rx: i16,
    pub thumb_ry: i16,
}

/// Table 0x1032a818: wButtons' mask to padsin's virtual number. The triggers (0x10000 / 0x20000 there) are the bytes.
const VIRTUAL: [(u16, u8); 14] = [
    (0x0001, 6),  // d-pad up
    (0x0002, 7),  // d-pad down
    (0x0004, 5),  // d-pad left
    (0x0008, 4),  // d-pad right
    (0x1000, 3),  // A
    (0x2000, 0),  // B
    (0x4000, 1),  // X
    (0x8000, 2),  // Y
    (0x0010, 14), // Start
    (0x0020, 15), // Back
    (0x0040, 10), // left thumb
    (0x0080, 13), // right thumb
    (0x0100, 8),  // left shoulder
    (0x0200, 11), // right shoulder
];

/// One XInput state as gamepad.rs reads a pad. FUN_10005a40: a trigger is its button held at any value but 0, and a
/// thumb is v x 127 >> 15 (so axes x 127 gives retail's -127 ..= 126 exactly); XInput's up is already positive.
pub fn pad_of(g: &XGamepad) -> Pad {
    let mut buttons: Vec<u8> = VIRTUAL.iter().filter(|(m, _)| g.buttons & m != 0).map(|(_, n)| *n).collect();
    buttons.extend([(g.left_trigger, 9), (g.right_trigger, 12)].into_iter().filter(|(v, _)| *v != 0).map(|(_, n)| n));
    buttons.sort_unstable();
    let thumb = |v: i16| ((v as i32 * 127) >> 15) as f32 / 127.0;
    Pad {
        buttons,
        axes: [
            thumb(g.thumb_lx),
            thumb(g.thumb_ly),
            thumb(g.thumb_rx),
            thumb(g.thumb_ry),
            g.left_trigger as f32 / 255.0,
            g.right_trigger as f32 / 255.0,
        ],
    }
}

/// Anything held, for the pad that plays being the last one touched (as gilrs's and SDL's).
fn touched(g: &XGamepad, prev: &XGamepad) -> bool {
    g.buttons & !prev.buttons != 0
        || (g.left_trigger != 0 && prev.left_trigger == 0)
        || (g.right_trigger != 0 && prev.right_trigger == 0)
}

#[cfg(windows)]
pub use dll::XInput;

#[cfg(windows)]
mod dll {
    use super::{XGamepad, pad_of, touched};
    use crate::client::gamepad::Pad;
    use std::time::{Duration, Instant};
    use windows_sys::Win32::Foundation::{FreeLibrary, HMODULE};
    use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

    /// XINPUT_STATE.
    #[repr(C)]
    #[derive(Default)]
    struct State {
        packet: u32,
        pad: XGamepad,
    }

    /// XINPUT_VIBRATION.
    #[repr(C)]
    struct Vibration {
        left: u16,
        right: u16,
    }

    type GetState = unsafe extern "system" fn(u32, *mut State) -> u32;
    type SetState = unsafe extern "system" fn(u32, *mut Vibration) -> u32;

    /// xinput1_3 first (retail's xinputdll.dll links it), then the newer ones. A shim may hook only one of them, so
    /// each that loads is kept and the first with a pad plays.
    const DLLS: [&str; 3] = ["xinput1_3.dll", "xinput1_4.dll", "xinput9_1_0.dll"];
    /// How often the slots with no pad are asked again (an empty slot is slow to ask on some systems).
    const SCAN: Duration = Duration::from_secs(1);
    const SLOTS: u32 = 4;

    struct Dll {
        name: &'static str,
        module: HMODULE,
        get: GetState,
        set: Option<SetState>,
    }

    impl Drop for Dll {
        fn drop(&mut self) {
            unsafe { FreeLibrary(self.module) };
        }
    }

    pub struct XInput {
        dlls: Vec<Dll>,
        /// The DLL whose pads play (index into `dlls`), and each slot's last state while it has a pad.
        cur: Option<usize>,
        slots: [Option<XGamepad>; SLOTS as usize],
        active: Option<u32>,
        next_scan: Instant,
        /// The slot the motors run on and their level.
        motor: Option<(u32, u16)>,
    }

    impl XInput {
        pub fn new() -> Result<XInput, String> {
            let mut dlls = Vec::new();
            for name in DLLS {
                let wide: Vec<u16> = name.encode_utf16().chain([0]).collect();
                let module = unsafe { LoadLibraryW(wide.as_ptr()) };
                if module.is_null() {
                    continue;
                }
                let get = unsafe { GetProcAddress(module, c"XInputGetState".as_ptr().cast()) };
                let set = unsafe { GetProcAddress(module, c"XInputSetState".as_ptr().cast()) };
                let Some(get) = get else {
                    log::info!("gamepad (XInput): {name} has no XInputGetState");
                    unsafe { FreeLibrary(module) };
                    continue;
                };
                // SAFETY: XInputGetState / XInputSetState have these signatures in every XInput DLL.
                type Proc = unsafe extern "system" fn() -> isize;
                let (get, set) = unsafe {
                    (std::mem::transmute::<Proc, GetState>(get), set.map(|s| std::mem::transmute::<Proc, SetState>(s)))
                };
                dlls.push(Dll { name, module, get, set });
            }
            if dlls.is_empty() {
                return Err("no XInput DLL".into());
            }
            let names: Vec<&str> = dlls.iter().map(|d| d.name).collect();
            log::info!("gamepad: XInput ({}) as the fallback", names.join(", "));
            Ok(XInput { dlls, cur: None, slots: [None; 4], active: None, next_scan: Instant::now(), motor: None })
        }

        fn get(&self, dll: usize, slot: u32) -> Option<XGamepad> {
            let mut s = State::default();
            // ERROR_SUCCESS; anything else (ERROR_DEVICE_NOT_CONNECTED) is no pad.
            (unsafe { (self.dlls[dll].get)(slot, &mut s) } == 0).then_some(s.pad)
        }

        /// Reads the pads: every frame the slots that have one, every SCAN the others (and, with none, the other DLLs).
        pub fn update(&mut self) {
            let now = Instant::now();
            let scan = now >= self.next_scan;
            if scan {
                self.next_scan = now + SCAN;
            }
            if self.slots.iter().all(Option::is_none) {
                if !scan {
                    return;
                }
                self.cur = None;
                self.active = None;
                for d in 0..self.dlls.len() {
                    if (0..SLOTS).any(|s| self.get(d, s).is_some()) {
                        self.cur = Some(d);
                        break;
                    }
                }
            }
            let Some(d) = self.cur else { return };
            for s in 0..SLOTS {
                let i = s as usize;
                if self.slots[i].is_none() && !scan {
                    continue;
                }
                let prev = self.slots[i];
                self.slots[i] = self.get(d, s);
                match (prev, self.slots[i]) {
                    (None, Some(_)) => {
                        log::info!("gamepad (XInput) {} connected through {}", s + 1, self.dlls[d].name);
                        self.active.get_or_insert(s);
                    }
                    (Some(_), None) => {
                        log::info!("gamepad (XInput) {} disconnected", s + 1);
                        if self.motor.is_some_and(|(m, _)| m == s) {
                            self.motor = None;
                        }
                    }
                    (Some(p), Some(g)) if touched(&g, &p) => self.active = Some(s),
                    _ => {}
                }
            }
            if self.active.is_none_or(|s| self.slots[s as usize].is_none()) {
                self.active = (0..SLOTS).find(|&s| self.slots[s as usize].is_some());
            }
        }

        /// The pad that plays, as gamepad.rs reads one.
        pub fn state(&self) -> Option<Pad> {
            self.active.and_then(|s| self.slots[s as usize]).map(|g| pad_of(&g))
        }

        /// The connected pads' names, the one that plays first.
        pub fn names(&self) -> Vec<String> {
            let Some(d) = self.cur else { return Vec::new() };
            let mut v: Vec<(bool, u32)> =
                (0..SLOTS).filter(|&s| self.slots[s as usize].is_some()).map(|s| (Some(s) != self.active, s)).collect();
            v.sort();
            v.into_iter().map(|(_, s)| format!("XInput controller {} ({})", s + 1, self.dlls[d].name)).collect()
        }

        /// Both motors at `level` on the pad that plays (FUN_10008ed0, XInputSetState); a pad that stops playing
        /// stops.
        pub fn motors(&mut self, level: u16) {
            let Some(d) = self.cur else { return };
            let Some(set) = self.dlls[d].set else { return };
            let target = self.active.filter(|_| level != 0);
            if let Some((s, l)) = self.motor {
                if Some(s) == target && l == level {
                    return;
                }
                if Some(s) != target {
                    unsafe { set(s, &mut Vibration { left: 0, right: 0 }) };
                    self.motor = None;
                }
            }
            if let Some(s) = target {
                unsafe { set(s, &mut Vibration { left: level, right: level }) };
                self.motor = Some((s, level));
            }
        }
    }

    impl Drop for XInput {
        fn drop(&mut self) {
            self.motors(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_and_triggers_as_retail() {
        // A, B, Start, d-pad up; a trigger at 1 of 255 is held (FUN_10005a40 tests the byte against 0).
        let g = XGamepad { buttons: 0x1000 | 0x2000 | 0x0010 | 0x0001, left_trigger: 1, ..Default::default() };
        let p = pad_of(&g);
        assert_eq!(p.buttons, vec![0, 3, 6, 9, 14]);
        let g = XGamepad { buttons: 0x0300 | 0x00c0 | 0x0020 | 0xc000, right_trigger: 255, ..Default::default() };
        assert_eq!(pad_of(&g).buttons, vec![1, 2, 8, 10, 11, 12, 13, 15]);
    }

    #[test]
    fn thumbs_as_retail() {
        // v x 127 >> 15, arithmetic: full up 126, full down -127, -1 to -1.
        let g = XGamepad { thumb_lx: 32767, thumb_ly: -32768, thumb_rx: -1, thumb_ry: 259, ..Default::default() };
        let a = pad_of(&g).axes;
        let back: Vec<i32> = a[..4].iter().map(|v| (v * 127.0).round() as i32).collect();
        assert_eq!(back, vec![126, -127, -1, 1]);
        assert_eq!((a[4], a[5]), (0.0, 0.0));
    }

    #[test]
    fn touched_is_a_new_press() {
        let held = XGamepad { buttons: 0x1000, ..Default::default() };
        assert!(touched(&held, &XGamepad::default()));
        assert!(!touched(&held, &held));
        assert!(touched(&XGamepad { left_trigger: 3, ..held }, &held));
    }
}
