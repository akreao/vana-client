## The account window: Change password (ours, 2026-11-10)

Issue lenonk/vana-client#150 (its other half, the trust token, is "Two-factor sign-in: Trust this computer"). Retail
changed the password in PlayOnline (the Viewer), never in FFXiMain; our login stands in for PlayOnline's.

- LSB's ChangePassword (command 0x30, src/login/auth_session.cpp, since 2020): `username` / `password` checked
  (validatePassword), then with a validated TOTP the `otp` too; a banned account refused; an empty `new_password`
  refused; the bcrypt hash written and every trust token of the account dropped; reply result 0x06. Every refusal is
  result 0x07 (a wrong password, a wrong code, an empty new one, a failed write). A server without the command logs
  "Unhandled auth code" and sends nothing.
- Asked of other servers (2026-11-10, one request each for a made-up account): HorizonXI and CatsEyeXI both ask for
  loader 2.0.x and then answer 0x07 as stock LSB does, so neither removed the command; whether a real account's change
  goes through there is not known (a handler always answering 0x07 would look the same).
- Ours: the login form's Two-factor button is now Account (focus n + 4, help "Change this account's password, or turn
  two-factor sign-in ... on or off"), its window titled Account, the menu Change password, Turn on two-factor, Turn
  off two-factor, New recovery code, Close. Change password: New password and Again (stars, plain ASCII, 32 at most,
  as the login form), Code (empty without two-factor sign-in), Change / Cancel; the current password is the form's.
  Checked before sending: a new password, and the two the same. 0x07 shows "Not changed: check the password in the
  login form, and the code if two-factor sign-in is on."; 0x06 "The password is changed: log in with the new one.",
  and the old one goes from the form's Password and from Remember password (the form's and the saved login's sealed
  copy), and the kept trust token with it.
- Seen live (lnxtest01, classic): a wrong form password gave the 0x07 line; a change to a new password, then a login
  with it to the world, then a change back and a login with the original.
