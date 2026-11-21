## Zone-in: no steps before the settle, and the settle snaps as FUN_1017a400 (2026-11-15)

- Reported: walking through the door from Ve'Lugannon Palace (177) into Ru'Aun Gardens (130) left the user floating
  under the island. LSB placed us at (145.6, -25.66, 404.5); the road there is at -24.0. The key was still held, and
  the steps ran on the new zone during the half second before it settled (falls were held, input steps were not),
  taking us to -23.0; the settle then looked only from 0.5 above the feet down (a step's floor ray), missed the road
  1.0 above and took the next face down, 3.08: the island's underside, 26 below.
- Now no input step runs while the zone-in is unsettled, and Ground::settle is FUN_1017a400's snap (from 2 above the
  feet down 50, else from 2 below up 15; nearest to the start), the ray retail puts entities on the floor with, then
  the nearest face. Test ruaun_zone_in_settles_on_the_road (-25.66, -24.5 and -23.0 all land on -24.0). The select
  screen ring (select.rs, settle from -1) renders the same.
