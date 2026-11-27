# Hotbars: macro bars by job

Hotbars is a bundled mod (from Arusia's VCHotbar, PR #88): up to three bars of up to twelve slots over the chat
window. Each slot holds a macro of as many lines as you like. Its action's icon shows on the slot (the first spell,
ability, weapon skill or item a line names), with a shade and the time left while it recasts; a slot with no action
shows its label. Turn it on in Config → Extras, on its page (Hotbars), where its size and place are set too.

## Sets

Every main / sub job pair has its own bars and crossbars: THF/NIN's are not THF/DNC's (and THF with no sub job has
its own). The first time you play a pair, its set starts from what Hotbars showed before sets went by sub job (the
job's main set with that sub job's set over it) less another sub job's actions: Utsusemi stays on THF/NIN and goes
from THF/DNC. A spell neither of your jobs learns is another job's; an ability is judged by the abilities the game
lists for you once the list has settled after the job change, and only at main level 99 (below that it may be your
main job's, not learned yet). Weapon skills, items, equipment, commands and typed macros always stay.

## On the bars

- **Keys:** bar 1 takes `1`..`0`, `-`, `=`; bar 2 the same with Ctrl, bar 3 with Alt. A slot shows its key in its
  corner. **Assign Keys** (right-click the bars) changes them: the slots' edges turn teal, the slot under the pointer
  (or clicked) is ringed in gold, and the next key combination pressed becomes its key. A combination something else
  takes (another slot, one of the game's keys such as its Ctrl macros, another mod, the client) asks first. Clicks
  and the bars' keys do nothing meanwhile; Escape, or Stop Assigning Keys in the right-click menu, ends it. The menu
  then also clears a slot's key, gives it back its bar's, or resets every key.
- **Click** a slot to run it; **right-click** anywhere on the bars for the menu (on a slot: clear it, too); **drag** a
  slot onto another to swap them (in the sub set when either comes from it).
- **Hover** a slot for its name and lines.
- The jobs over the top bar open the manager.

The game's macro palette hides while the bars show (Hide Stock Macro Bar); its Ctrl and Alt keys still run its
macros unless a bar uses them.

## Macros

Each line runs as typed. `/wait 2` alone on a line waits two seconds; `<wait 2>` at the end of a line waits after it.
A macro still running ignores a second press.

## The library and the Builder

Each main job has a **library** of macros (`//hb`). The Library page has two tabs: the current job's (**THF Macros** on
THF; New, Edit, Delete, Copy From) and **Shared Macros** (New, Edit, Delete). Each shows its macros as tiles: click one to
choose it, right-click to edit it, drag it onto a slot of the bars to put it there. A slot can name a library macro
instead of holding its copy, so editing the macro changes every slot (main or sub set) that names it. **Copy From**
picks another job that has macros and shows its macros as tiles under the job's: click one, or drag it onto the job's tiles
(modern UI), to copy it to the current job's library, or **Copy All** for every one (a macro it already has is dimmed and skipped); **Close** puts them
away.

The **Builder** makes one. New or Edit opens it in the window's place; Save keeps the macro in the library and
Cancel goes back to it. Pick a type (spell, trust, job ability, weapon skill, pet command, item, equipment, a system
command such as `/attack`, `/heal`, `/assist` or `/lockon`, or Macro for typed lines), search, pick and choose the
target. The name follows what you pick until you type your own, and so does the **icon** beside it: click the icon to
choose any spell, ability, weapon skill, pet command, status or held item's icon for the macro (a Macro of typed
lines too); No Icon goes back to its action's. A slot of one action runs it as the game's menus do
(the magic menu's checks for a spell, and so on), not as a typed line; **Macro** is for lines of your own
(`/p Casting <t> | /wait 2 | /echo done`), run as typed commands. A macro saved before with lines after its action
opens as a Macro of all its lines. System commands show a game icon standing in for them (a sword for
`/attack`, Regen for `/heal`). Spells are the ones the magic list offers: learned, and castable by your main or
support job at its level (blue magic only while set). The help line describes the spell, ability or item under the
cursor or the pointer.

Abilities that open a list in the game's menu (Rune Enchantment, Ward, Effusion, Phantom Roll, Quick Draw, Sambas,
Waltzes, Jigs, Steps, the Flourishes, Stratagems, Blood Pact: Rage and Ward, Ready) open the same list here, marked
`›`: pick Rune Enchantment, then Ignis. Back returns to the list. A search finds the abilities in those lists too.

**Group slots:** one slot for several actions. Open a list such as Rune Enchantment, or search (`cure` finds Cure to
Cure VI), and press **Make a Group of These**; Save it and put it on a slot as any macro. The slot shows the member
used last, with a small arrow in its corner. Pressing it (its key or a click) opens the members in a list over the
slot, the one used last under the cursor: Up / Down and Enter, or a click, runs one; the number keys pick one too;
Escape or a click elsewhere closes it. While you edit a group, the list shows its members (click one to take it
out); search to add more, of any type. The Target menu sets every member's target. Groups are optional: one slot per
rune still works as before.

**Shared macros:** turn **Shared** on in the Builder (New on the Shared Macros tab starts with it on) and the macro is
in every job's library, on the Shared Macros tab (a teal edge marks its tile), on every character and world, for a
Warp Ring or a mount you want on any job. Edit or delete it from any job; a slot of any job can name it. Turn Shared
off and it goes to your current job, with a copy for each other job of this world that has it on a slot. Two clients
open at once share the shared macros: each change takes turns on `sticky.json` and starts from what the other saved,
and the other client shows it after `//hb reload` or a relog.

The macros are kept in `mod-data/hotbar/files/macros-<World>.json` (`macros.json` until the world is known), and
shared ones in `sticky.json` beside it, with ids `s1`, `s2`, ...; a slot names one as `{ "ref": "s1" }`:

```json
{
  "WAR": {
    "library": { "1": { "name": "Provoke", "action": { "kind": "ja", "name": "Provoke", "target": "t" },
                        "lines": ["/p Provoked <t>"] },
                 "2": { "name": "Ignis", "action": { "kind": "ja", "name": "Ignis", "target": "me",
                                                     "line": "/ja \"Ignis\" <me> <wait 1>" } } },
    "main": { "1": { "1": { "ref": "1" }, "2": { "name": "Berserk", "lines": ["/ja \"Berserk\" <me>"] } } },
    "sub": { "NIN": { "1": { "6": { "name": "Utsusemi", "lines": ["/ma \"Utsusemi: Ichi\" <me>"] } } } }
  }
}
```

## Commands

- `//hb` opens or closes the manager.
- `//hb show`, `//hb hide`: the bars for this session.
- `//hb run <bar> <slot>`: runs a slot (for a key bind).
- `//hb reload`: reads the macro files again.
- `//hb wipestock` empties the game's macro sets after keeping a copy; `//hb restorestock` puts it back.
