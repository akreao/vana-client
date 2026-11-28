# Retail Macros

Brings your macro books over from a game install on this computer, such as the retail client's. `//retailmacros`
opens it.

## Finding the install

The **Install** list holds the installs found: on Windows from the registry (where the PlayOnline installer records
FINAL FANTASY XI), on Linux and macOS from Wine and Steam (Proton) prefixes. Pick one, and the client asks whether the
mod may read that folder; it reads nothing before you allow it, and changes nothing there. **Choose a Folder...** opens
the system's folder dialog instead: pick the `FINAL FANTASY XI` folder (the one holding `USER`), the `USER` folder, or
one character's folder.

An install marked **(this client's)** is the one this client plays from: its macro books are already yours.

## Books

The **Character** list holds the install's character folders (each named by its content id, with the book titles it
has); the one with this character's id is chosen to begin with. The list below shows its 40 books, how many macros each holds,
and the Hotbars job its macros would go to. Click a book to pick it (again to leave it out); **Pick All** picks all 40.

- **Copy to Macro Palettes** copies the picked books, all ten sets and their titles, over this character's books of the
  same numbers (Ctrl / Alt palettes, Edit Macro). The books they replace are kept in the character's
  `USER/<content id>/backup/` folder first; the chat log says where.
- **Send to Hotbars** adds the picked books' macros to Hotbars' library, under each book's job (from its title: "WAR",
  "Ninja"; else your main job). **Hotbars Job for This Book** changes it, or sends it to the shared library. A macro
  Hotbars already has is skipped. Hotbars must be on.

Lines that run `/lua` are left out either way: a `/lua` line is yours to write.

## Permissions

`files.outside` (reading the folders you pick or allow) and `actions` (copying the books when you press the button).
