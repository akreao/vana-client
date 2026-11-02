## Remaining retail windows (ours, 2026-11-02)

- link11 (naming a new linkshell, linkshell.rs Naming) takes the modern window (modern/social.rs naming): the
  Linkshell icon (item 0x201) tinted by linkshell::tint, the name as a kit field (a click is the name row's Enter:
  typing, linkshell::text_events as before), Red / Green / Blue as 16-stop sliders (linkshell::naming_channel sets
  and keeps the channel, ending a different one's edit), OK off while the name is empty and Cancel through
  naming_pick (retail's Enter on child 5 / any other). Retail's keys and help bar still apply. Under Modern,
  linkshell::draw and pointer no longer run at all, so the retail link6 header stops drawing over the modern link12.
- Play Time (statcom2 child 9) shows on the modern Status page live: textcmd::played / playtime_text split out of
  /playtime (FUN_1007cab0, group 10 0x2e / 0x2f), Enter or the button still prints the line.
- Region Info's map windows (region.rs draw_windows: conquer1, conq6sta, beseige1 / 2, coloresu, colorank, the
  legend) are one modern panel (modern/regioninfo.rs) at the screen's edge away from the cursor, from
  region::info_view: retail's label sprites named by their captions (framesus nations 73..95, keytops3 levels 77..81,
  Al Zahbi 370+, armies 367+, strongholds 374+, orders 377+, alliance 119) with English fallbacks; the gauges as
  bars. Coalition names in colorank are ours (*inf:* the Coalition list's order); retail draws them as one sprite.
  The map image, its markers and the cursor stay retail's draw.
