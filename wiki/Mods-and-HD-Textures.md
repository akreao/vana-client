# Mods and HD Textures

vana-client has two kinds of mods, which share one folder (`~/.local/share/vana-client/mods/<id>/`, Windows `%APPDATA%\vana-client\mods`). One mod folder can hold both.

| Kind | What it is | Page |
|---|---|---|
| DAT overlay mods | Replacement game files: HD textures, cursors, other DATs | this page |
| Lua mods | Scripts that add windows, commands, trackers and automation | [[Lua Mods]] |

## The mod manager

**Config > Settings > Mods** installs and manages DAT overlay packs:

- **Install** a pack from an archive (zip, 7z or rar) found in your Downloads folder or the mods folder
- **Order** packs (a later pack wins where two replace the same file)
- **Enable / disable** and **uninstall** them

Nothing in an archive is run. The manager only takes DAT files at game paths (XIPivot's layout: `ROM*/<dir>/<file>.DAT`) and cursor files at the game root. DAT overlays are merged per chunk, so two packs that change different parts of one file both apply.

## The HD texture pack

The project's HD pack is the game's textures upscaled 4x. Download it from the [hd-pack-1 release](https://github.com/lenonk/vana-client/releases/tag/hd-pack-1), then install the archive from the Mods page.

## Making a texture pack

The client has command-line tools for the whole pipeline:

```sh
vana-client tex-export --out export/                     # every stock image once, as PNGs plus manifest.tsv
# ...upscale or edit the PNGs in export/ with your tool of choice, into upscaled/...
vana-client tex-pack --manifest export/manifest.tsv --upscaled upscaled/ --out pack/   # rebuild the DATs, as DXT
vana-client tex-slim --pack pack/ --out slim/            # keep only the images that differ from stock
vana-client mod-install --archive my-pack.zip --root test-mods/   # install the archive as the Mods page would
```

Run `vana-client <command> --help` for each command's options. `tex-dump` saves the images of particular DATs, and `status-dump` the status icons.
