#!/usr/bin/env bash
# A build with NVIDIA DLSS (cargo feature `dlss`, render/upscale/dlss.rs; docs/upscaler-engine.md). The feature links
# NVIDIA's NGX library, so cargo needs NVIDIA's DLSS SDK (github.com/NVIDIA/DLSS at v310.9.1, the tag dlss_wgpu 6.0 is
# made for; NVIDIA RTX SDKs License, not in this repository). This fetches the part the build needs (the headers and
# this platform's NGX library, under 1 MB) once, into <target dir>/dlss-sdk, sets DLSS_SDK and VULKAN_SDK, and runs
# `cargo build --features dlss` with any arguments given (default --release).
#   release/build-dlss.sh                  target/release/vana-client
#   release/build-dlss.sh --profile dist   target/dist/vana-client
# DLSS_SDK set beforehand: that SDK checkout is used instead (checked, never fetched into or changed). VULKAN_SDK: the Vulkan headers' prefix (default /usr,
# where libvulkan-dev / vulkan-headers put them). Needs git, clang (bindgen) and the Vulkan headers besides the
# usual build tools. The DLSS runtime itself is not needed to build: the client downloads it on first use.
set -euo pipefail
cd "$(dirname "$0")/.."
tag=v310.9.1
case "$(uname -s)" in
    Linux) lib="/lib/Linux_$(uname -m)/libnvsdk_ngx.a" ;;
    MINGW* | MSYS* | CYGWIN*) lib="/lib/Windows_x86_64/x64/" ;;
    *) echo "DLSS builds on Linux and Windows only" >&2; exit 1 ;;
esac
if [ -n "${DLSS_SDK:-}" ]; then
    # The caller's own SDK checkout: checked, never fetched into or changed.
    sdk=$DLSS_SDK
    [ -f "$sdk/include/nvsdk_ngx.h" ] ||
        { echo "DLSS_SDK=$sdk has no include/nvsdk_ngx.h: point it at a DLSS SDK checkout, or unset it" >&2; exit 1; }
else
    # The script's own cache in the target dir, fetched (again) while it lacks the headers.
    target=${CARGO_TARGET_DIR:-target}
    [[ $target == /* ]] || target=$(pwd)/$target
    sdk=$target/dlss-sdk
    if [ ! -f "$sdk/include/nvsdk_ngx.h" ]; then
        echo "fetching the DLSS SDK $tag (headers and ${lib#/}) into $sdk"
        rm -rf "$sdk"
        git clone --quiet --depth 1 --branch "$tag" --filter=blob:none --no-checkout https://github.com/NVIDIA/DLSS.git "$sdk"
        # The patterns through stdin: Git Bash would turn "/include/" given as an argument into a Windows path.
        printf '%s\n' /include/ "$lib" /LICENSE.txt | git -C "$sdk" sparse-checkout set --no-cone --stdin
        git -C "$sdk" -c advice.detachedHead=false checkout --quiet
    fi
fi
command -v clang >/dev/null || { echo "clang is needed (bindgen): install clang / libclang-dev" >&2; exit 1; }
export DLSS_SDK=$sdk VULKAN_SDK=${VULKAN_SDK:-/usr}
[ -f "$VULKAN_SDK/include/vulkan/vulkan.h" ] || [ -f "$VULKAN_SDK/Include/vulkan/vulkan.h" ] ||
    { echo "no Vulkan headers under VULKAN_SDK=$VULKAN_SDK: install libvulkan-dev / vulkan-headers, or set VULKAN_SDK" >&2; exit 1; }
[ $# -gt 0 ] || set -- --release
exec cargo build --features dlss "$@"
