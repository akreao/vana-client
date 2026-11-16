<!-- dll-build: 30261006_0 -->
## No c2s 0x015 under the control lock: the airship berth doors (2026-11-16)

- Report: on retail, Port Jeuno's departures door for Windurst (`_6ue`, event 39) played the walk through the door and
  then left us where the event started. LSB lets us through.
- Event 39 (actor 0x010f605f): choice, `cm54` walk, `fdo0`, op 0x47 sub 0 to (3.064, 8.001, -59.25), the 0x05C round
  trip, `fdi0`. Our 0x05C is byte for byte what FUN_100b62b0 / FUN_100a05e0 build (x val@2, y val@6, z val@4,
  dir val@8). The retail server's answer (capture, two tries) is s2c 0x065 mode 1 plus 0x05B mode 0 at our stored
  place, to 0.001, with our facing: the pre-event spot. FUN_1009d500 case 1 puts that into our event body and the end
  keeps it (FUN_10089a60), so we stood back at the door. LSB answers with the 0x05C's own coordinates, so it worked
  there.
- Why the server had the old place: we sent c2s 0x015 every datagram through the event, the one carrying the 0x05C
  too, all at the pre-event place (our place waits while bound). Retail sends none: FUN_100983f0 builds the 0x015
  only while the control lock DAT_10482f18 is clear (0x10098530), and the event start (0x1009533e) and a talk request
  (0x1007c17b) set it. So the 0x05C is the only position the server has from us during the event.
- Ours: SelfState::locked mirrors World::control_lock each frame, and the net thread queues no 0x015 while it holds.
  The first datagram after the lock goes carries the place the event left us at.
