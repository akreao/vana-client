## Event scenes: data/weat, the first-folder fallback, the default env (2026-11-15)

- Some event-only scenes keep their envs one folder deeper, under `<zone>/data/weat/<tag>` (516 and 517, the Rise of
  the Zilart ending, event 7 of zone 163; ordinary zones have `<zone>/weat/<tag>`). Our parser read that `data` folder
  as a scenery area named "data", so the scene had no env of its own and drew unlit: 516's hills black against a
  plain blue clear colour. Now `data` is the scene's own.
- FUN_1018a7a0 picks the weather folder: the asked weather, else `suny`, else (FUN_10072290 on `weat`) the env keys
  of the whole `weat` folder, nearest at or before the time; with every key at 0000 that is the first folder. 516
  has only `rain` / `thdr` (and the event asks for weather 2 then), so it takes rain: dark, fog to about 290.
  ZoneWeather::first.
- A scene with no `weat` folder at all is lit by DAT_1065ca58 as FUN_1018b170 fills it (FUN_101880c0's payload
  without a weather object): grey 0x80 light and fog, fog 80..100, a black clear colour, draw distance 100, no dome
  (Env::retail_default, ZoneWeather::retail_default); ours drew such a scene unlit. zone-shot takes it too.
- Test event_scene_weather. Live (Lnxcsfix, event 7 replayed in zone 163): 516's storm plain textured and lit, 517
  clean.
