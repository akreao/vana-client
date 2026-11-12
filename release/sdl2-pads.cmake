# sdl2-sys's bundled SDL (the Linux pad fallback, client/gamepad_sdl.rs), cut to what pads need: no audio, video or
# render (SDL 2.26's PipeWire audio no longer builds against current PipeWire headers, and none of it is used).
# Passed to sdl2-sys as SDL2_TOOLCHAIN from .cargo/config.toml.
set(SDL_AUDIO_ENABLED_BY_DEFAULT OFF)
set(SDL_VIDEO_ENABLED_BY_DEFAULT OFF)
set(SDL_RENDER_ENABLED_BY_DEFAULT OFF)
set(CMAKE_C_STANDARD 99)
