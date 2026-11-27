## Message 8 by the windows' frame hooks (2026-11-27)

<!-- dll-build: 30261102_0 -->

Modlog-gaps item 194 (split from 182). Addresses are 30261102_0, decompiled with Ghidra; FUN_10160fd0's order, the
equipment and item-list hooks and FUN_1013a650 checked in Rizin. The windows are named by `menu_classes.json`.

- FUN_10160fd0 (the window manager's frame) runs the mouse routine FUN_101267f0 (call at 0x1016137a: the hover tail
  on the focused window) and then calls every open window's vtable +0x28 (0x101613cc / 0x101613d5) with 1 for the
  focused window DAT_10622a44 and 0 for the others. So the hooks see the focus the buttons and the hover left.
- 25 functions call FUN_10127770 (mouse +0x4d == 2, the mouse driving the menus, and FUN_10087070(0) clear). In 22 of
  them a focused window asks FUN_1011a0a0 (window +0x3a .. +0x40, both ends in) about another window's frame and sends
  a message; a resting pointer sends it again every frame:

  | Hook (focused window) | Other window's frame | Message |
  | --- | --- | --- |
  | win_equip_v28 0x10134030 (menu equip; only with the list open, FUN_10132f40) | inventor | 8 |
  | FUN_101f3d70 (inventor) | itmsortw, or itmsort2 when inventor +0x90 | 8 |
  | | bank | 4 |
  | | equip | 8 |
  | win_bank_v50 0x101de050 | itmsortw / inventor | 8 / 3 |
  | FUN_101174b0 / FUN_10117d90 (fe_equip / fe_inven) | the other | 8 |
  | win_bluequip_v50_2 0x101e0d00 / FUN_101e1780 (bluinven for bluequip) | bluinven / bluequip | 8 |
  | win_automato_v28 0x10209050 / win_automato_v60 0x1020a510 (bluinven for automato) | bluinven / automato | 8 |
  | win_cmbhlst_v50 0x101e4200 | cmbhwin | 8 |
  | win_loot_v28 0x1013ffe0 (loot +0x164 = lnowin) | lnowin | 8 |
  | win_trade_v28 0x10152e80 / FUN_1013a540 (gift) | gift / trade | 8 |
  | win_mcr1edit_v28 0x101a61d0 / win_mcr2edit_v28 0x101a68b0 | the other palette | FUN_101a5b10(own column) |
  | | mcrbedit | 8 |
  | win_macro_v28 0x101a6fd0 (cursor on lines 4..9) | mcrledit | 8 |
  | FUN_101a7510 (mcrbedit) / FUN_101a78b0 (mcrledit) | macro | 8 |
  | win_magic_v60 0x1022e1a0 (list +0xc0) | its sort window | 8 |
  | win_ability_v60 0x10225c50 / win_emote_v60 0x102275a0 / win_mount_v60 0x10228180 | abisortw | 8 |
  | FUN_10242e30 (lobby, vtable 0x10336b40) | chfwin | 8 |

  The others: FUN_102055f0 (from FUN_10205290, walks a window table with FUN_101603a0), FUN_10139070 (from
  FUN_10138c40: FUN_1011a0e0 against a scroll area, mouse +0x4e) and FUN_10127770's own definition.
- No hook asks about menu inspect (the mannequin's grid, rmlo4_case 0x10135a70; win_inspect_v28 0x10135260) or
  mcresed (the equipment-set grid, win_mcresed_v28 0x10136d20), and the item list asks only about itmsortw /
  itmsort2, bank and equip. So the pointer moves no focus between the mannequin's or an equipment set's grid and
  its list. Ours did (on pointer moves); now neither does.
- FUN_101a5b10: both palettes' columns (+0xd, +0xe) to the focused palette's column, the focus (+0xc) to the other
  palette (FUN_1015f920), SE 0001. Ours took the column under the pointer.
- Message 8 of trade (win_trade_onkey 0x10152bb0 case 8) and of gift (FUN_1013a650 case 8): SE 0013 (FUN_10037e90)
  and the other window raised, in any trade state; ours had no message 8 there (Num+ now sends it too). The same
  handlers play SE 0013 on trade's Down onto gift (message 1) and gift's Up onto trade (message 2), where ours played
  0001; gift's message 6 puts trade's cursor on Cancel and raises trade with no sound (ours played 0003).

Ours: `winhits::hook` (once a retail frame of ours, `Hover::hook_tick`; the pointer shown, `textcmd::busy` clear,
inside the frame both ends in), asked after each window's hover and buttons in equipwin, bluset, pupset, treasure,
synth, magic, celestite, macroedit and trade. The windows' own rules for message 8 stay in their `num_plus` (a pick
held refuses every retail frame in bluequip / automato / fe_*).

Not built (rows 226..228): the storage view's and the item list's hooks (itmsortw / itmsort2 / bank, messages 8 / 3 /
4; after the storage view's mouse of row 193), the lobby's chfwin pair, FUN_102055f0 / FUN_10139070.

Tests: `client::winhits::tests::hooks_once_a_retail_frame`. Not checked on screen.
