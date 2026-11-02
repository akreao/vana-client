## Pending actions (ours, 2026-11-02)

- Retail shows nothing between sending an action and the server's answer. Ours keeps the last 0x01A (magic 3, weapon
  skill 7, job ability 9) or 0x037 sent; the first 0x028 with our id as actor confirms it, a 0x029 with our id as
  actor before that is taken as the refusal (but experience, limit and capacity points, defeats: 6, 8, 105, 253, 371,
  372, 718, 735). The heuristic leans on LSB answering a refused action with a 0x029 and an accepted one with a 0x028
  first.
