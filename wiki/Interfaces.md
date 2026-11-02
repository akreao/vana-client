# Interfaces: Classic and Modern

vana-client has two interfaces. Switch at any time in **Config > Settings > Client > Interface** (config field `modern_ui`). Classic is the default.

| Modern | Classic |
|---|---|
| ![Items in the modern interface](https://raw.githubusercontent.com/akreao/vana-client/master/docs/screenshots/modern-items.jpg) | ![Items in the classic interface](https://raw.githubusercontent.com/akreao/vana-client/master/docs/screenshots/classic-items.jpg) |
| ![Config in the modern interface](https://raw.githubusercontent.com/akreao/vana-client/master/docs/screenshots/modern-config.jpg) | ![Config in the classic interface](https://raw.githubusercontent.com/akreao/vana-client/master/docs/screenshots/classic-config.jpg) |

## Classic

The retail client's windows, drawn from the game's menu files the way the original draws them, with the retail key set: numpad movement, autorun, first person, F-key targeting, macros and palettes.

## Modern

A new interface over the same game rules, built with egui on the client's wgpu renderer. While Modern is on, it draws the windows it has and leaves out the retail windows they replace; everything else stays retail until it has a modern counterpart.

- Movable windows with layouts saved per character, a command menu, tooltips, a minimap and a chat window with tabs
- Mouse support everywhere, but never required: every action has a keyboard path and a visible focus
- Keyboard-first combat; directional menus and Enter / Escape stay first-class
- Equipment by slot from every bag: pick a slot and see every piece the character can wear there, wherever it is
- Numbered containers merged (Mog Wardrobe 1..8 is one Mog Wardrobe)
- Jobs written as `WAR22/WHM11`

### Keys in Modern windows

Every modern page uses the same key model:

| Key | Does |
|---|---|
| Up / Down | Move between controls (wrapping) and along a list |
| Left / Right | Change a value, walk a row of buttons, page a list or turn tabs |
| Tab / Shift+Tab | Turn a page's tabs from anywhere |
| Enter | Step an option or press |
| Escape | Back; in a yes / no prompt, No |

With the mouse, hovering only highlights a control; a click moves the cursor there and activates it; a drag moves a slider.

### Mouse in the world

- Left click selects a target. Double click on an actor talks to an NPC or checks anyone else.
- Right click on an actor opens its menu (a monster: Target, Attack, Check; a player: Target, Check, Tell, Invite to Party, Trade; an NPC: Talk, Target, Trade). Item and party rows have right-click menus too.
- Ctrl + left click on the ground walks there.
- Mog House storage: click a row, or drag it onto the other pane, to move the stack.
- In both interfaces, a right drag turns the camera and the character, and the wheel zooms the camera.

The modern UI's full plan, decisions and state are in [docs/modern-ui.md](https://github.com/akreao/vana-client/blob/master/docs/modern-ui.md).
