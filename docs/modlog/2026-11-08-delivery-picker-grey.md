## Delivery Box picker: Ex by augment and the worn linkshell (2026-11-08)

- The send picker's row colour is LAB_1012e1a0 (30261006_0; a callback, not a Ghidra function): 0x80404040 grey or
  0x80808080. Ex by augment = FUN_101cb960 / FUN_101cba20 (FUN_101cb8a0 / FUN_101cb990 on the slot's extdata) found
  and its flag byte has bit 2, as the item window's Ex mark.
- Item flag 0x10 (CanSendAccount): grey to another account (+0x1f2 clear); on our own account grey when Ex by augment
  and item flag bit 0 is clear. Without 0x10: grey with 0x2000 (NoDelivery) or Ex by augment.
- Then grey when equipped (FUN_10131f10) or when the row is the shell worn as linkshell 1: FUN_100e9730(0, &bag, &slot,
  1) with the slot and the list's container (vt +0x54) matching. Linkshell 2 is not tested here.
- Ours: post::picker_tint (augment::marked_with, World.aug_no_mark copied from GameUi's augment tables). Not seen live
  (needs an augmented Ex item or a worn linkshell in the send picker).
