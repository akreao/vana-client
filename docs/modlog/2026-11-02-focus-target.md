## Modern focus and last target (ours, 2026-11-02)

- Not retail: World.last_target is the target before the current one, set in target::set (the sub-target prompt's
  stand-ins don't count; us neither). target::last_target gives it while it is here and targetable.
- Modern only (the classic UI keeps retail's tags): `<lt>` names the last target in text and resolves to it in
  target commands; `<ft>` is `<focust>`.
- The modern focus panel (hud.rs others_panel, place "focus" on the target panel): a row for the focus target
  (retail's /focustarget, a member of our six) and one for the last target, each with its HP; a click targets it.
  Hidden when neither is set.
