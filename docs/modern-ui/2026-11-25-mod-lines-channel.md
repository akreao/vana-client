- Chat line channel and mods (2026-11-25, ours): a line a mod runs by itself with `commands.run` (a timer, a reply to
  the server; not on the player's key or click) does not become the chat line's default channel, and does not end the
  player's one-off Shout or Message mode either. The player's own sends, macros, key binds and a mod's line run on the
  player's input (Key Commands' saved binds, a hotbar click) still do.
- Older bullet made wrong: "A line sent on Say, Party, Linkshell, Linkshell 2 or Unity becomes the default channel"
  now holds for all but a mod's own background lines.
