## The Mog House door menu drawn again (2026-11-05)

- The classic UI never drew "mogdoor" (the exit menu when the house's exits are unlocked: Stay, Area you entered
  from, Change floors, Select an area): myroom::WINDOW was the 16-byte padded "menu    mogdoor ", while layouts are
  kept trimmed (dat::ui::norm_name), so the window lookup, its children (the mouse rows) and the drawing all missed.
  The keys still worked, so the menu answered while invisible. The name is now trimmed, and UiData::menu trims the
  name it is given, so a padded name finds its layout. The Modern UI draws its own list and was not affected.
