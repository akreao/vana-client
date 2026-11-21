# Gamepad

Controllers work on Linux and Windows: any pad with an SDL mapping (Xbox, PlayStation, Switch Pro and most others). They go through the retail client's own pad path: buttons set the pad bits the game's key functions test, and the sticks fill its stick bytes.

Button settings use the format of retail's registry value `padsin000` (written by FFXiPadConfig.exe), so a retail setting can be copied over as is. The default is FFXiPadConfig's XInput layout, the same as the official PC layout.

## Default layout

| Button | Does |
|---|---|
| A / Cross | Confirm, the action menu |
| B / Circle | Cancel |
| X / Square | Main menu |
| Y / Triangle | Active window |
| D-pad | In a menu: the cursor. In the field and in a target prompt: up / down the party and alliance members (wrapping), left / right the actors on screen |
| Left stick / right stick | Move / camera (the sticks never drive menus) |
| LB / L1 | Autorun; in a window with tabs, the previous tab |
| RB / R1 | Open the chat line; with the windows hidden, a screenshot; in a window with tabs, the next tab |
| LT / L2, RT / R2 | In the field: open or close the Ctrl / Alt macro palette (d-pad left / right picks the macro, up / down the set, A runs it, B closes). In a window: held, the d-pad pages lists; both held, left / right switch the item bag. In a Geo- spell's target prompt: held, the d-pad moves where it lands |
| L3 | Rest, or lock on with a target; on the map, back to where you were |
| R3 | First / third person |
| Back / Select | Hide or show the windows |
| Start | Log out |

## Settings

| Field | Default | Meaning |
|---|---|---|
| `gamepad` | `true` | Read a game controller |
| `gamepad_padsin` | empty (FFXiPadConfig's XInput default) | Button settings in `padsin000` format: 32 numbers, one per function, each an XInput button number (0 B, 1 X, 2 Y, 3 A, 4 / 5 / 6 / 7 d-pad right / left / up / down, 8 LB, 9 LT, 10 L3, 11 RB, 12 RT, 13 R3, 14 Start, 15 Back), an axis 32 + n (1 left X, 2 left Y, 4 right X, 5 right Y; negative inverts) or -1 |
| `gamepad_padexsin` | empty | `padexsin000`: the map used while the "alternate map" button is held |
| `gamepad_deadzone` | `0.33` | Stick travel ignored around the centre (retail's 42 of 127), measured round so the stick does not snap to its axes |
| `gamepad_invert_y` | `false` | Invert the camera's up and down |

The function order for `gamepad_padsin` is in [padsin_order.md](https://github.com/akreao/vana-client/blob/master/docs/re/gamepad/padsin_order.md); the reverse-engineering notes are in [docs/re/gamepad](https://github.com/akreao/vana-client/blob/master/docs/re/gamepad/findings.md).

## Typing on the pad

RB opens retail's software keyboard ("menu    keypadus": its pages, keys, pad buttons, placement and sounds) when
Config > Misc. 4 > Software Keyboard is on, typing into the open field, or into the chat line when no field is open.
It is drawn larger in the classic UI and as a panel in the modern UI. With the option off, RB does nothing here.
