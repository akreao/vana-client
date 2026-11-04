## Real character shadows fade with the caster's height (2026-11-04, ours)

- Not retail. A character on high ground (atop a wall) cast a full shadow onto the ground far below. The receive pass
  (render/sunshadow.rs) now reads the four map texels of each PCF tap, compares each one and fades it by its own caster's height, then blends them (a filtered comparison would leave a high caster's unfaded edge). Its shadow is full while the caster is
  within 3 world units of the surface along the light, and fades out by 7 units, so feet and body shadows are unchanged.
- Test: render::sunshadow::receive_darkens_shadowed_floor (a caster just above darkens, one 20 units above does not).
