- Click-to-move (2026-11-02, ours; one way to try the open question, which stays open for upstream): with the modern UI,
  Ctrl + left click on the ground (no actor under the pointer) walks there. The point is the pointer's ray on the
  collision mesh; the way is an A* search on a one-yalm grid (a floor within 0.8 up / 1.5 down of the last, no wall at
  chest height), straightened where a straight walk stays on the floor, within 60 yalms (client/clickmove.rs). The walk
  is autorun steered at each point as /follow steers it, so autorun's rules hold; a move or turn key, the autorun key,
  arriving, or 1.5 s without half a yalm of progress ends it. No way found: the error sound. Plain clicks are as before.
