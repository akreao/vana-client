## First start on a low-spec GPU: cheaper defaults (2026-11-20)

Not retail (settings defaults). A GPU on the GL / GLES backend, one without BC textures (mobile and SBC GPUs: Raspberry
Pi, Mali, Adreno) or a software rasterizer counts as low-spec (`render::GpuClass`). The first time one is seen
(`low_spec_seen`), each of these still at its default takes the cheaper value (`Config::low_spec_defaults`):

- Bump mapping (0017) off: FFXiMain's own default.
- Anisotropic filtering 1 (retail samples without it), frame limit 30 (retail's rate), Draw Distance Retail.
- Background resolution: half the window on a software rasterizer (as before), three quarters on a low-spec GPU
  whose window is over 960 wide. The UI stays at the window's size.
- A window larger than its screen (640x480 / 720x720 handhelds) goes borderless at the screen's size.

Desktop GPUs on Vulkan / DX12 / Metal (all have BC) keep every default. Settings the player changed are kept.
Measured on llvmpipe (zone pass, 1280x720, Bastok Markets / Konschtat): defaults 34.6 / 34.7 ms, bump off 27.3 /
28.2, bump off + anisotropy 1 21.3 / 19.0, background halved 18.6 / 16.1.
