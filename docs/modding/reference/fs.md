# `fs`

`fs` reads and writes files in the mod's data folder: `mod-data/<mod id>/files` in the client's data folder (on Linux
`~/.local/share/vana-client/mod-data/<mod id>/files`). Every write goes there. Reads (`fs.read`, `fs.exists`,
`fs.list`) also see the files that came with the mod, in its folder, as a read-only fallback: the data folder
first, then the mod's files, so a mod can ship default data and save the user's copy over it. A path whose first
part is `shared` (`"shared/notes.txt"`) is in the shared area instead, a folder every mod with the `files.shared`
permission can read and write. The calls work anywhere (also in the lobby and in filters) and take effect at once.

```lua
local raw = fs.read("config.json")          -- the saved copy, else the one shipped with the mod
local cfg = raw and json.decode(raw) or {}
cfg.runs = (cfg.runs or 0) + 1
fs.write("config.json", json.encode(cfg))   -- always saved in the data folder
```

Paths are relative and use `/` (a `\` works too). A leading `/` and `.` parts are ignored, so `"/a/./b.txt"` is
`"a/b.txt"`. A path with a `..` part raises "outside the mod's folder", and one with a `:` in a part raises "not a
relative path". A path through a symbolic link (or a Windows junction), in the data folder or in the mod's files,
raises "a link, which could lead outside the mod's folder"; the data folder itself may be one. Errors from the system
come as "<path>: <reason>".

## Functions

| Function | What it does |
| --- | --- |
| [`fs.read(path)`](#fsread) | A file's contents, or nil. |
| [`fs.write(path, data)`](#fswrite) | Writes a file, replacing what was there. |
| [`fs.append(path, data)`](#fsappend) | Adds to the end of a file. |
| [`fs.exists(path)`](#fsexists) | Whether a file or folder is there. |
| [`fs.list(path?)`](#fslist) | The names in a folder. |
| [`fs.mkdir(path)`](#fsmkdir) | Makes a folder. |
| [`fs.remove(path)`](#fsremove) | Removes a file, or a folder with everything in it. |
| [`fs.rename(from, to)`](#fsrename) | Moves or renames a file or folder. |
| [`fs.lock(path, fn)`](#fslock) | Runs a function holding a lock on a file, so clients take turns. |
| [`fs.game_installs()`](#fsgame_installs) | The game installs on this computer. |
| [`fs.pick_folder(title?, handler)`](#fspick_folder) | Lets the player pick a folder the mod may read. |
| [`fs.open_folder(path, reason, handler)`](#fsopen_folder) | Asks the player to let the mod read a folder. |
| [`fs.read_outside(path)`](#fsread_outside) | A file's contents in a folder the player let the mod read. |
| [`fs.exists_outside(path)`](#fsexists_outside) | Whether a file is there, in such a folder. |
| [`fs.list_outside(path)`](#fslist_outside) | The names in such a folder. |

Folders outside the mod's (the `_outside` calls) need the `files.outside` permission, and are read only. The mod
reads only a folder the player picked in the folder dialog or allowed when the client asked (`fs.open_folder`), until
the mod unloads. Paths there are full paths. The client's mod data is never one of them: read it with `fs.read`.

## `fs.read`

```lua
fs.read(path) -> string?
```

Reads a whole file. It looks in the data folder (or the shared area) first; when the file is not there, it reads the
mod's file at the same path.

**Parameters**

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `path` | `string` | yes | The file's relative path. |

**Returns** `string?`: the file's contents (any bytes), or `nil` when neither place has the file.

**Needs** the `files.shared` permission for a path under `shared/`.

**Errors** A refused path (see above); "this needs the files.shared permission: ..." for `shared/` without the
permission; "<path>: <reason>" when the file is there but cannot be read (a folder, for example).

**Example**

```lua
local notes = fs.read("notes.txt")
if notes then
  for line in notes:gmatch("[^\n]+") do print(line) end
else
  print("No notes yet.")
end
```

Files read as they are, text or binary, from the data folder and from the mod's folder alike.

## `fs.write`

```lua
fs.write(path, data)
```

Writes `data` to a file in the data folder (or the shared area), replacing the file if it exists. Missing folders on
the way are made.

**Parameters**

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `path` | `string` | yes | The file's relative path. |
| `data` | `string` | yes | What to write (any bytes). |

**Returns** nothing.

**Needs** the `files.shared` permission for a path under `shared/`.

**Errors** A refused path; "this needs the files.shared permission: ..." for `shared/` without the permission;
"<path>: <reason>" when the system refuses the write.

**Example**

```lua
local p = game.player()
fs.write("last_zone.txt", tostring(p.zone))
fs.write("logs/" .. os.date("%Y-%m-%d") .. ".txt", "started\n")
```

## `fs.append`

```lua
fs.append(path, data)
```

Adds `data` to the end of a file in the data folder (or the shared area), making the file and its folders when they
are missing. Good for logs.

**Parameters**

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `path` | `string` | yes | The file's relative path. |
| `data` | `string` | yes | What to add (any bytes). |

**Returns** nothing.

**Needs** the `files.shared` permission for a path under `shared/`.

**Errors** As [`fs.write`](#fswrite).

**Example**

```lua
events.on("chat_message", function(line, sender)
  fs.append("chat.log", os.date("%H:%M ") .. sender .. ": " .. text.plain(line) .. "\n")
end)
```

## `fs.exists`

```lua
fs.exists(path) -> boolean
```

Whether a file or folder with this path is in the data folder (or the shared area) or among the mod's files.

**Parameters**

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `path` | `string` | yes | The relative path. |

**Returns** `boolean`: `true` when either place has it.

**Needs** the `files.shared` permission for a path under `shared/`.

**Errors** A refused path; "this needs the files.shared permission: ..." for `shared/` without the permission.

**Example**

```lua
if not fs.exists("data") then
  fs.mkdir("data")
end
```

## `fs.list`

```lua
fs.list(path?) -> string[]
```

The names in a folder, sorted, with a `/` at the end of each folder's name. The data folder's names and the mod's
are merged (each name once). For a path under `shared/`, only the shared area is listed.

**Parameters**

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `path` | `string` | no, default `""` | The folder's relative path; none lists the top. |

**Returns** `string[]`: the names (not paths). A folder that does not exist gives an empty list.

**Needs** the `files.shared` permission for a path under `shared/`.

**Errors** A refused path; "this needs the files.shared permission: ..." for `shared/` without the permission.

**Example**

```lua
for _, name in ipairs(fs.list("logs")) do
  if name:sub(-1) ~= "/" then
    print(name)
  end
end
```

## `fs.mkdir`

```lua
fs.mkdir(path)
```

Makes a folder in the data folder (or the shared area), with any missing folders above it. A folder that already
exists is not an error.

**Parameters**

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `path` | `string` | yes | The folder's relative path. |

**Returns** nothing.

**Needs** the `files.shared` permission for a path under `shared/`.

**Errors** A refused path; "this needs the files.shared permission: ..." for `shared/` without the permission;
"<path>: <reason>" when the system refuses.

**Example**

```lua
fs.mkdir("cache/maps")
fs.write("cache/maps/readme.txt", "made by " .. mod.name)
```

## `fs.remove`

```lua
fs.remove(path)
```

Removes a file, or a folder with everything in it, from the data folder (or the shared area). Nothing there is not
an error. The mod's files are never removed.

**Parameters**

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `path` | `string` | yes | The relative path. |

**Returns** nothing.

**Needs** the `files.shared` permission for a path under `shared/`.

**Errors** A refused path; "fs.remove \"\": name a file or folder inside the data folder" for `""`, `"/"`, `"."` or
`"shared"` (the whole data folder or the whole shared area); "this needs the files.shared permission: ..." for
`shared/` without the permission; "<path>: <reason>" when the system refuses.

**Example**

```lua
for _, name in ipairs(fs.list("cache")) do
  fs.remove("cache/" .. name)
end
```

## `fs.rename`

```lua
fs.rename(from, to)
```

Moves or renames a file or folder within the data folder (or between it and the shared area). Missing folders above
`to` are made; a file already at `to` is replaced.

**Parameters**

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `from` | `string` | yes | The relative path of what to move. |
| `to` | `string` | yes | Its new relative path. |

**Returns** nothing.

**Needs** the `files.shared` permission when either path is under `shared/`.

**Errors** A refused path; "this needs the files.shared permission: ..." for `shared/` without the permission;
"<from>: <reason>" when the move fails (for example when `from` does not exist).

**Example**

```lua
local tmp = "save.json.tmp"
fs.write(tmp, json.encode(storage.account))
fs.rename(tmp, "save.json")
```

Only files in the data folder can be moved: to copy one of the mod's files there, `fs.write` what `fs.read`
returns.

## `fs.lock`

```lua
fs.lock(path, fn) -> any
```

Runs `fn` while holding an operating-system lock on the file at `path` in the data folder (or the shared area). The
file is made empty if it is missing. Two clients started from the same install share the data folder, so a mod that
reads a file, changes it and writes it back can lose the other client's change. Doing the read and the write inside
`fs.lock` makes the clients take turns. The lock is let go when `fn` returns or raises an error, and when the client
exits. Lock a file kept only for locking (`"save.json.lock"`), not the one being written: on Windows a locked file
cannot be written through another handle.

When another client holds the lock, the call waits for it, up to 2 seconds, then raises an error. Nothing else runs
while it waits, so keep `fn` to a read and a write. A nested `fs.lock` on the same file from within `fn` waits on
itself and fails: run the inner code directly.

**Parameters**

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `path` | `string` | yes | The relative path of the lock file. |
| `fn` | `function` | yes | Called with no arguments while the lock is held. |

**Returns** what `fn` returns.

**Needs** the `files.shared` permission when `path` is under `shared/`.

**Errors** A refused path; "fs.lock <path>: held by another client" after 2 seconds of waiting; "<path>: <reason>" when
the file cannot be made or locked; any error `fn` raises, passed on.

**Example**

```lua
fs.lock("counter.json.lock", function()
  local t = json.decode(fs.read("counter.json") or "{}")
  t.n = (t.n or 0) + 1
  fs.write("counter.json", json.encode(t))
end)
```

Older clients have no `fs.lock`; a mod listed in the catalogue checks for it (`if fs.lock then ... end`).

## `fs.game_installs`

```lua
fs.game_installs() -> table[]
```

The retail game installs on this computer: on Windows from the registry (`HKLM\SOFTWARE\PlayOnline{US,,EU}\InstallFolder`,
as the game reads it), elsewhere from the same key in Wine prefixes (`WINEPREFIX`, `~/.wine`, Steam's prefixes) and the
installer's default folders in them. Finding an install does not let the mod read it: ask with `fs.open_folder`.

**Returns** `table[]`: one table per install:

| Field | Type | Description |
| --- | --- | --- |
| `path` | `string` | The `FINAL FANTASY XI` folder. |
| `source` | `string` | Where it was found: the registry key, or the prefix's `system.reg`. |
| `own` | `boolean` | Whether this client plays from it too (its macros are already the client's). |

**Needs** the `files.outside` permission.

**Example**

```lua
for _, i in ipairs(fs.game_installs()) do
  print(i.path .. (i.own and " (this client's)" or ""))
end
```

## `fs.pick_folder`

```lua
fs.pick_folder(title?, handler) -> nil
```

Shows the system's folder dialog. The folder the player picks may be read from then on; `handler` gets its full path,
or nil when the dialog was closed.

**Parameters**

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `title` | `string` | no | The dialog's title (default "Choose a folder"). |
| `handler` | `function` | yes | Called with the path, or nil. |

**Returns** nothing.

**Needs** the `files.outside` permission.

**Example**

```lua
fs.pick_folder("Choose your FINAL FANTASY XI folder", function(path)
  if path then print(#fs.list_outside(path) .. " names there") end
end)
```

## `fs.open_folder`

```lua
fs.open_folder(path, reason, handler) -> nil
```

Asks the player, in a window of the client's (the mod cannot change it or answer it), to let the mod read a folder
and everything in it. `handler` gets true when the player allows it, false otherwise. A folder already allowed (or
inside one) answers true at once.

**Parameters**

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `path` | `string` | yes | The folder's full path. |
| `reason` | `string` | yes | Why the mod wants it, shown under the path. |
| `handler` | `function` | yes | Called with true or false. |

**Returns** nothing.

**Needs** the `files.outside` permission.

**Errors** `"<path>": not a folder`; `"<path>": the mod data is read with fs.read` for the client's mod data (or a
folder holding it).

**Example**

```lua
fs.open_folder("/home/me/.wine/drive_c/Program Files (x86)/PlayOnline", "To read your macros.", function(ok)
  print(ok and "allowed" or "not allowed")
end)
```

## `fs.read_outside`

```lua
fs.read_outside(path) -> string?
```

Reads a whole file in a folder the player picked or allowed.

**Parameters**

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `path` | `string` | yes | The file's full path. |

**Returns** `string?`: the file's bytes, or nil when it is not there.

**Needs** the `files.outside` permission.

**Errors** `"<path>": not a full path` (a relative path, or one with a `..` part); `"<path>": not in a folder you
picked or allowed` (a link that leads out of one counts as outside); "<path>: <reason>" when it cannot be read.

**Example**

```lua
local raw = fs.read_outside(folder .. "/mcr.ttl")
```

## `fs.exists_outside`

```lua
fs.exists_outside(path) -> boolean
```

Whether a file or folder is there, in a folder the player picked or allowed.

**Parameters**

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `path` | `string` | yes | The full path. |

**Returns** `boolean`.

**Needs** the `files.outside` permission.

**Errors** As `fs.read_outside`.

**Example**

```lua
if fs.exists_outside(install .. "/USER") then print("this install has characters") end
```

## `fs.list_outside`

```lua
fs.list_outside(path) -> string[]
```

The names in a folder the player picked or allowed, sorted; folders end in `/`.

**Parameters**

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `path` | `string` | yes | The folder's full path. |

**Returns** `string[]`.

**Needs** the `files.outside` permission.

**Errors** As `fs.read_outside`, and "<path>: <reason>" when it is not a folder.

**Example**

```lua
for _, name in ipairs(fs.list_outside(install .. "/USER")) do print(name) end
```
