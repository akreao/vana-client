## Arrows turn the camera while the target prompt is up (2026-11-10)

Issue lenonk/vana-client#195.

- Picking a spell's or ability's target left the arrows dead: the camera axes (Input::axis) need Shift whenever a
  window is active, and the magic / ability list hidden behind the prompt (and the action menu) still counts as one.
- Retail's camera axis callbacks (FUN_10124770 / FUN_101248e0, 30261102_0) skip the window gate while the 3D prompt
  is up (target object DAT_10579924 +0xc4, set by FUN_1015aa10), unless the chat line is being typed in
  (FUN_101b7260: +0xf108 is 4 / 5). The party-list cursor (+0x59) gets no such exception.
- Ours: input::Frame.prompt is set while `w.prompt` is up and no party-list cursor is, and the arrows reach the camera
  when not typing and there is no window, Shift is held, or the prompt is up. The party-list cursor's Up / Down are
  untouched.
