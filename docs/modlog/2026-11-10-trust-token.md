## Two-factor sign-in: Trust this computer (ours, 2026-11-10)

Issue lenonk/vana-client#150. Adds to "Two-factor sign-in".

- LSB's Login (src/login/auth_session.cpp, otp_helpers.h) takes `trust_token` and `trust_this_computer`. For an
  account with a validated TOTP it tries a non-empty token first (SHA-256 of it looked up in accounts_trust_tokens for
  the account, unexpired). A token it turns down with no `otp` to fall back on ends in result 0x13; with an `otp`, the
  code decides. After a login verified either way, `trust_this_computer` gets a new 64-character token in the reply
  (`trust_token`), stored hashed with a 30-day expiry. Tokens are not used up by a login; a password change and
  RemoveTotp drop all of the account's, and a banned account's go at its next login.
- Ours (not retail): "Trust this computer" right of the one-time code on the login form (focus n + 6, Tab after the
  code; Right from the code, Left back, Down to Remember password; not for PlayOnline). The code field narrows to a
  code's width there; its hint reads "trusted" while a token is kept. Ticked, a login sends the kept token, and asks
  for a new one only when a code was typed (so a token runs out 30 days after the code login that got it, not later).
- The token is sealed as the remembered password is (secret.rs, ChaCha20-Poly1305 under secret.key), bound to the
  server and account under a "trust" prefix so a password and a token never open as each other. Config
  `trust_computer` / `trust_token`, and the same on each saved login, so the box and token go with the account (and
  with a forgotten login). The form's pair follows its server and account: when they change (typed, picked,
  forgotten) or the form first shows, they are the saved login's; with none the token goes and the box stays as it
  was; a token this computer's key cannot open (a replaced secret.key) is dropped there and then.
- It is kept as soon as the reply brings it, whatever the lobby step after does. Unticking the box drops it at once;
  so do a 0x13 refusal (its message "the saved trust token was rejected; enter your one-time code" shows, the box
  stays ticked so the next code login gets a new one) and Turn off in the Two-factor window.
- Seen live (lnxtest01 with a TOTP secret set in accounts_totp, removed afterwards): a code login with the box ticked
  got a token (one row, 30 days); the modern form then showed "trusted" and logged in on the password alone; with the
  row deleted the next login gave 0x13 and the kept token went; a code login got a new one; unticking by keys (Down,
  Right, Space from Password) dropped it. After review: the hint went to "only with 2FA" with one letter of the
  account deleted and back to "trusted" when retyped; with secret.key replaced the form dropped the kept token at once.
