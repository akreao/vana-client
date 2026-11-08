# Sourced by the build scripts. build_cpus: the cores a release build may use, as a comma list for taskset -c or
# docker --cpuset-cpus: the ones this shell may run on (its affinity, not 0..nproc), less the last 8 (left to the LSB
# map server and its bots) when more than 8 remain; with 8 or fewer, all of them, so a small host still builds.
build_cpus() {
    local list ids=() r a b i
    list=$(awk '/^Cpus_allowed_list:/ { print $2 }' /proc/self/status 2>/dev/null || true)
    [ -n "$list" ] || list="0-$(($(nproc) - 1))"
    IFS=, read -ra ranges <<<"$list"
    for r in "${ranges[@]}"; do
        a=${r%-*}
        b=${r#*-}
        for ((i = a; i <= b; i++)); do ids+=("$i"); done
    done
    local n=${#ids[@]}
    local keep=$((n > 8 ? n - 8 : n))
    local IFS=,
    echo "${ids[*]:0:keep}"
}
