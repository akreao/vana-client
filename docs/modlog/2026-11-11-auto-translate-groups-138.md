## Auto-translate: the Items and Key Items groups, variable blocks (2026-11-11)

<!-- dll-build: 30261102_0 -->

Modlog-gaps item 138 ("Auto-translate: the item / key item groups built at run time (the 200-record item cache);
kind 6; the nested FD 06 block"). Addresses are 30261102_0 (MODLOG "Auto-translate: the dictionary" + 0x15d0 here:
FUN_101bb2b0 -> FUN_101bc880, FUN_101bb680 -> FUN_101bcc50, FUN_101bb740 -> FUN_101bcd10, FUN_101bc100 ->
FUN_101bd6d0, FUN_101ac5e0 -> FUN_101adbb0, FUN_101ac990 -> FUN_101adf60).

- The item record cache (DAT_10602620, 200 records of 0x294 bytes; id +0, lock byte +0x1b3, name +0x1b4). Every item
  record lookup goes through FUN_1015d210(id): a held id answers from the cache (FUN_1015b580), else the record is
  read from its file into the next slot (FUN_1015b5c0: the index DAT_106229cc steps first and skips locked slots, so
  slot 1 fills first and slot 0 last). The name is the record's first string (parser field +0x14c8 =
  FUN_10258b20(rec, 0)), copied with strncpy(.., 0x20) and cut at +0x1d2: 30 bytes. About 35 functions call it (bags,
  equipment, shops, the auction house, /translate, /itemsearch, the phrase resolver itself).
- Ours: `dat::items::Items` keeps the ring (`Recent`, ids and names only). `Items::item` takes a slot,
  `Items::touch` counts a lookup one of our own caches answered (`client::items::Items::item`, `Items::noun`),
  `Items::item_raw` reads without it (the GM picker's scan over every id). Which lookups ours makes is ours' own
  windows' doing, so the cache's contents can differ from retail's in detail.
- An item phrase's text is that cache name: "Bronze Sword", not the log noun "bronze sword" ours showed
  (`NameSource::item_name`). Resolving one takes a cache slot, so an item seen in chat joins the Items group.
- FUN_101bcd10 builds both groups again whenever the root category list opens: FUN_101bd060 (the query) calls it
  only with no word and no category, so a typed word completes against the groups as they stood at the last such
  open, and before the first one there are none. The group count D+0x224 goes back to the file's count D+0x228,
  then Key Items (FUN_101bcc50, slot D+0x218) and then Items (FUN_101bc880, slot D+0x214) each take the next slot
  when they have an entry. So the categories are the 42 file groups, Key Items, Items; a group with no entry is not
  listed. (This settles the order MODLOG left inferred.)
  - Items: every cache slot with an id, minus names starting '.' and ids 0x7400..0xfffe (FUN_101bc6d0); entry =
    4-byte id, length, name; the word's key is its text. Header kind 7, title / key group 0xb 0x91 / 0x92.
  - Key Items: the key item table in file order (FUN_10155fb0 count, FUN_10155fc0(i) id, FUN_10155b20(i) name), the
    owned ones (FUN_1022c600 -> FUN_10097e50: bit id & 31 of DAT_10481068[id >> 5], the s2c 0x055 bits), ending at
    the first owned id 0x181..0x183, 0xfe at most. Header kind 0x13, title / key 0x93 / 0x94.
  - Ids never hold a 00 byte: base kind, + 1 for id 0 (FF FF), + 2 when the high byte is 0 (FF lo), + 3 when the
    low byte is 0 (hi FF).
- A category is found by FUN_101bc640: a file group by its group byte, else a built group by its raw kind byte (7 /
  0x13). The list's Enter (FUN_101ae200) treats a candidate as a category when its id's last byte is 0, which the
  00-free ids above never are. Ours tested `kind < 7` as well, which kept the two built categories from opening.
- Resolver FUN_101bd6d0: the kind is folded to 7 (7..0x12) or 0x13 (0x13..) first (checked in Rizin at 0x101bd73f;
  the jump table's cases 0xb / 0xf with slots D+0x220 / D+0x21c are dead). The id comes from the raw kind: 7 / 0xb /
  0xf / 0x13 both bytes, 9 / 0xd / 0x11 / 0x15 the low byte, 0xa / 0xe / 0x12 / 0x16 the high byte, any other kind 0.
  Item id 0 is the Items group's title only while that group stands, else nothing (the line ends); key item id 0
  the Key Items title while that stands, else key item 0's name. Ours gave the titles always and read kinds past
  0x16 as ids.
- Variable blocks (kind 6), FUN_101adbb0: `FD 06 x x x nn <nn bytes> x`, nn + 7 bytes, nn a signed byte (over 0x7f
  the line ends). The nn bytes are a template copied out as they are; each FD in it starts a 6-byte place. For a
  place the function calls itself on the bytes after the variable block (0x101add61: source + consumed): a block
  there, of any kind, is expanded without brackets or colour in the place and used up; with no block there the
  place's own four id bytes go to the resolver. A following block that fails ends the line; a place's own id that
  fails adds the resolver's -1 to the output length (the byte before it goes). The variable block itself gets no
  brackets. No code in the DLL writes one (no FD / 06 pair among its seven 0xFD stores).
- Ours: `font::decode_sjis` keeps a variable block's bytes one char a byte (so `encode_sjis` gives them back),
  `autotrans::Dictionary::expand` / `variable` expand it; `Dictionary::rebuild` / `built` are the two groups, rebuilt
  by `fep`'s query on the root list.
- Not built: the cache's lock byte (transient, during the scans) and FUN_1015b5c0's second argument (a record handed
  in by the caller reuses its slot). The Japanese / French second keys do not apply to these groups (entries of
  kinds 7..0x12 never have one; key item entries are loaded by FUN_101bbcb0 with the name as text and key).
- Tests: `dat::autotrans::tests::built_groups`, `variable_block`, `english_dictionary` (Bronze Sword, the group
  from the cache), `dat::items::tests::record_cache_ring`. Not checked live.
