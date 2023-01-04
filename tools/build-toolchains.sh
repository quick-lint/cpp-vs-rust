#!/usr/bin/env bash
set -e
set -u
set -o pipefail

root="$(cd "$(dirname "${0}")/.." && pwd)"

clean=1

toolchains_dir=/home/strager/Toolchains
rustc_dir=/home/strager/tmp/Projects/rust
llvm_project_dir="${rustc_dir}/src/llvm-project"

logs_dir="${toolchains_dir}/build-logs"

clang_profile_dir="${llvm_project_dir}/clang-profdata"
rustc_profile_dir="${rustc_dir}/rustc-rustc-profdata"

clang_ldflags="-fuse-ld=lld -Wl,-q"
clang_common_flags="-march=native -mtune=native"
clang_instr_flags="-Xclang -mllvm -Xclang -vp-counters-per-site=4"

main() {
    #build_clang_stage1
    #build_clang_stage2            # needs build_clang_stage1
    #build_clang_stage3_instr      # needs build_clang_stage2
    #profile_clang_pgo             # needs build_clang_stage3_instr
    #build_clang_stage4_qljs       # needs build_clang_stage2 and profile_clang_pgo
    #build_clang_stage4_qljs_boltinstr   # needs build_clang_stage4_qljs
    profile_clang_bolt            # needs build_clang_stage4_qljs_boltinstr
    build_clang_stage4_qljs_bolt  # needs build_clang_stage4_qljs and profile_clang_bolt

    #build_rustc_stage2            # needs build_clang_stage2
    #build_rustc_stage3_instr      # needs build_rustc_stage2
    #profile_rustc_pgo             # needs build_rustc_stage3_instr
    #build_clang_stage4_rustcpgo   # needs build_clang_stage2 and profile_rustc_pgo
    #build_rustc_stage4_pgo        # needs build_rustc_stage2 and build_clang_stage4_rustcpgo and profile_rustc_pgo
    ##build_rustc_stage4_pgo_boltinstr        # needs build_rustc_stage4_pgo
    ##profile_rustc_bolt            # needs build_rustc_stage4_pgo_boltinstr
    ##build_rustc_stage4_pgo_bolt   # needs profile_rustc_bolt
}

build_clang_stage1() {
    (
        cd "${llvm_project_dir}"
        rm_if_should_clean -rf build-stage1
        cmake \
            -S llvm \
            -B build-stage1 \
            -DCMAKE_BUILD_TYPE=Release \
            -DCMAKE_INSTALL_PREFIX="${toolchains_dir}/clang-stage1" \
            -DLLVM_TARGETS_TO_BUILD=X86 \
            -DLLVM_ENABLE_PROJECTS='clang;compiler-rt;lld' \
            -DLLVM_BINUTILS_INCDIR=/usr/include \
            -DLLVM_INCLUDE_TESTS=NO \
            -DLLVM_INCLUDE_UTILS=YES \
            -DLLVM_INSTALL_UTILS=YES \
            -G Ninja
        ninja -C build-stage1
        rm -rf "${toolchains_dir}/clang-stage1"
        ninja -C build-stage1 install
    ) |& log_output build_clang_stage1
}

build_clang_stage2() {
    (
        cd "${llvm_project_dir}"
        rm_if_should_clean -rf build-stage2
        PATH="${toolchains_dir}/clang-stage1/bin:${PATH}"
        cmake \
            -S llvm \
            -B build-stage2 \
            -DLLVM_BUILD_INSTRUMENTED=OFF \
            -DLLVM_BUILD_RUNTIME=YES \
            -DCMAKE_C_COMPILER="${toolchains_dir}/clang-stage1/bin/clang" \
            -DCMAKE_CXX_COMPILER="${toolchains_dir}/clang-stage1/bin/clang++" \
            -DCMAKE_AR="${toolchains_dir}/clang-stage1/bin/llvm-ar" \
            -DCMAKE_RANLIB="${toolchains_dir}/clang-stage1/bin/llvm-ranlib" \
            -DCMAKE_C_FLAGS="${clang_common_flags}" \
            -DCMAKE_CXX_FLAGS="${clang_common_flags}" \
            -DCMAKE_EXE_LINKER_FLAGS="${clang_ldflags}" \
            -DCMAKE_SHARED_LINKER_FLAGS="${clang_ldflags}" \
            -DCMAKE_MODULE_LINKER_FLAGS="${clang_ldflags}" \
            -DCMAKE_BUILD_TYPE=Release \
            -DCMAKE_INSTALL_PREFIX="${toolchains_dir}/clang-stage2" \
            -DLLVM_TARGETS_TO_BUILD=X86 \
            -DLLVM_ENABLE_PROJECTS='clang;compiler-rt;bolt;lld' \
            -DLLVM_ENABLE_RUNTIMES='libcxx;libcxxabi;libunwind' \
            -DLLVM_ENABLE_LTO=Full \
            -DLLVM_BINUTILS_INCDIR=/usr/include \
            -DLLVM_INCLUDE_TESTS=NO \
            -DLLVM_INCLUDE_UTILS=YES \
            -DLLVM_INSTALL_UTILS=YES \
            -DCMAKE_BUILD_WITH_INSTALL_RPATH=YES \
            -DLLVM_PARALLEL_LINK_JOBS=3 \
            -G Ninja
        # HACK(strager): This build often OOMs when linking. On failure, build again
        # with no parallelization.
        ninja -C build-stage2 -k0 || :
        ninja -C build-stage2 -j3
        rm -rf "${toolchains_dir}/clang-stage2"
        ninja -C build-stage2 install
    ) |& log_output build_clang_stage2
}

build_clang_stage3_instr() {
    (
        cd "${llvm_project_dir}"
        rm_if_should_clean -rf build-stage3-instr
        PATH="${toolchains_dir}/clang-stage2/bin:${PATH}"
        cmake \
            -S llvm \
            -B build-stage3-instr \
            -DLLVM_BUILD_INSTRUMENTED=IR \
            -DLLVM_PROFILE_DATA_DIR="${clang_profile_dir}" \
            -DLLVM_BUILD_RUNTIME=NO \
            -DCMAKE_C_COMPILER="${toolchains_dir}/clang-stage2/bin/clang" \
            -DCMAKE_CXX_COMPILER="${toolchains_dir}/clang-stage2/bin/clang++" \
            -DCMAKE_AR="${toolchains_dir}/clang-stage2/bin/llvm-ar" \
            -DCMAKE_RANLIB="${toolchains_dir}/clang-stage2/bin/llvm-ranlib" \
            -DCMAKE_C_FLAGS="${clang_common_flags} ${clang_instr_flags}" \
            -DCMAKE_CXX_FLAGS="${clang_common_flags} ${clang_instr_flags}" \
            -DCMAKE_EXE_LINKER_FLAGS="${clang_ldflags}" \
            -DCMAKE_SHARED_LINKER_FLAGS="${clang_ldflags}" \
            -DCMAKE_MODULE_LINKER_FLAGS="${clang_ldflags}" \
            -DCMAKE_BUILD_TYPE=Release \
            -DCMAKE_INSTALL_PREFIX="${toolchains_dir}/clang-stage3-instr" \
            -DLLVM_TARGETS_TO_BUILD=X86 \
            -DLLVM_ENABLE_PROJECTS='clang;compiler-rt;lld' \
            -DLLVM_ENABLE_RUNTIMES='libcxx;libcxxabi;libunwind' \
            -DLLVM_BINUTILS_INCDIR=/usr/include \
            -DLLVM_INCLUDE_TESTS=NO \
            -DLLVM_INCLUDE_UTILS=YES \
            -DLLVM_INSTALL_UTILS=YES \
            -DCMAKE_BUILD_WITH_INSTALL_RPATH=YES \
            -G Ninja
        ninja -C build-stage3-instr
        rm -rf "${toolchains_dir}/clang-stage3-instr"
        ninja -C build-stage3-instr install
    ) |& log_output build_clang_stage3_instr
}

profile_clang_pgo() {
    (
        rm -f "${clang_profile_dir}"/*.profraw
        PATH="${toolchains_dir}/clang-stage3-instr/bin:${PATH}"
        "${root}/tools/build-for-instr-cpp.sh"
    ) |& log_output profile_clang_pgo
}

build_clang_stage4_qljs() {
    (
        cd "${llvm_project_dir}"
        rm_if_should_clean -rf build-stage4-qljs
        PATH="${toolchains_dir}/clang-stage2/bin:${PATH}"
        "${toolchains_dir}/clang-stage2/bin/llvm-profdata" merge --output "${clang_profile_dir}/profdata" "${clang_profile_dir}"/*.profraw
        cmake \
            -S llvm \
            -B build-stage4-qljs \
            -DLLVM_PROFDATA_FILE="${clang_profile_dir}/profdata" \
            -DCMAKE_C_COMPILER="${toolchains_dir}/clang-stage2/bin/clang" \
            -DCMAKE_CXX_COMPILER="${toolchains_dir}/clang-stage2/bin/clang++" \
            -DCMAKE_AR="${toolchains_dir}/clang-stage2/bin/llvm-ar" \
            -DCMAKE_RANLIB="${toolchains_dir}/clang-stage2/bin/llvm-ranlib" \
            -DCMAKE_C_FLAGS="${clang_common_flags}" \
            -DCMAKE_CXX_FLAGS="${clang_common_flags}" \
            -DCMAKE_EXE_LINKER_FLAGS="${clang_ldflags}" \
            -DCMAKE_SHARED_LINKER_FLAGS="${clang_ldflags}" \
            -DCMAKE_MODULE_LINKER_FLAGS="${clang_ldflags}" \
            -DCMAKE_BUILD_TYPE=Release \
            -DCMAKE_INSTALL_PREFIX="${toolchains_dir}/clang-stage4-qljs" \
            -DLLVM_TARGETS_TO_BUILD=X86 \
            -DLLVM_ENABLE_PROJECTS='clang;compiler-rt;lld' \
            -DLLVM_ENABLE_RUNTIMES='libcxx;libcxxabi;libunwind' \
            -DLLVM_ENABLE_LTO=Full \
            -DLLVM_BINUTILS_INCDIR=/usr/include \
            -DLLVM_INCLUDE_TESTS=NO \
            -DLLVM_INCLUDE_UTILS=YES \
            -DLLVM_INSTALL_UTILS=YES \
            -DCMAKE_BUILD_WITH_INSTALL_RPATH=YES \
            -DLLVM_PARALLEL_LINK_JOBS=3 \
            -G Ninja
        # HACK(strager): This build often OOMs when linking. On failure, build again
        # with no parallelization.
        ninja -C build-stage4-qljs -k0 || :
        ninja -C build-stage4-qljs -j3
        rm -rf "${toolchains_dir}/clang-stage4-qljs"
        ninja -C build-stage4-qljs install
    ) |& log_output build_clang_stage4_qljs
}

build_clang_stage4_qljs_boltinstr() {
    (
        in_dir="${toolchains_dir}/clang-stage4-qljs"
        out_dir="${toolchains_dir}/clang-stage4-qljs-boltinstr"

        rm -rf "${out_dir}"
        cp -a "${in_dir}" "${out_dir}"

        cd "${out_dir}"
        for exe in bin/clang-15 bin/llvm-ar; do
            "${toolchains_dir}/clang-stage2/bin/llvm-bolt" \
                "${in_dir}/${exe}" \
                -instrument \
                --instrumentation-file="${clang_profile_dir}/$(basename "${exe}").bolt.fdata" \
                -o "${out_dir}/${exe}"
        done
    ) |& log_output build_clang_stage4_qljs_boltinstr
}

profile_clang_bolt() {
    (
        rm -f "${clang_profile_dir}/perf.data"
        PATH="${toolchains_dir}/clang-stage4-qljs-boltinstr/bin:${PATH}"
        "${root}/tools/build-for-instr-cpp.sh"
    ) |& log_output profile_clang_bolt
}

build_clang_stage4_qljs_bolt() {
    (
        in_dir="${toolchains_dir}/clang-stage4-qljs"
        out_dir="${toolchains_dir}/clang-stage4-qljs-bolt"

        rm -rf "${out_dir}"
        cp -a "${in_dir}" "${out_dir}"

        for exe in bin/clang-15 bin/llvm-ar; do
            "${toolchains_dir}/clang-stage2/bin/llvm-bolt" \
                "${in_dir}/${exe}" \
                -o "${out_dir}/${exe}" \
                -data="${clang_profile_dir}/$(basename "${exe}").bolt.fdata" \
                -dyno-stats \
                -icf=1 \
                -reorder-blocks=ext-tsp \
                -reorder-functions=hfsort+ \
                -split-all-cold \
                -split-functions \
                -use-gnu-stack
        done
    ) |& log_output build_clang_stage4_qljs_bolt
}

build_clang_stage4_rustcpgo() {
    (
        cd "${llvm_project_dir}"
        rm_if_should_clean -rf build-stage4-rustcpgo
        PATH="${toolchains_dir}/clang-stage2/bin:${PATH}"
        "${toolchains_dir}/clang-stage2/bin/llvm-profdata" merge --output "${rustc_profile_dir}/profdata" "${rustc_profile_dir}"/*.profraw
        cmake \
            -S llvm \
            -B build-stage4-rustcpgo \
            -DLLVM_PROFDATA_FILE="${rustc_profile_dir}/profdata" \
            -DCMAKE_C_COMPILER="${toolchains_dir}/clang-stage2/bin/clang" \
            -DCMAKE_CXX_COMPILER="${toolchains_dir}/clang-stage2/bin/clang++" \
            -DCMAKE_AR="${toolchains_dir}/clang-stage2/bin/llvm-ar" \
            -DCMAKE_RANLIB="${toolchains_dir}/clang-stage2/bin/llvm-ranlib" \
            -DCMAKE_C_FLAGS="${clang_common_flags}" \
            -DCMAKE_CXX_FLAGS="${clang_common_flags}" \
            -DCMAKE_EXE_LINKER_FLAGS="${clang_ldflags}" \
            -DCMAKE_SHARED_LINKER_FLAGS="${clang_ldflags}" \
            -DCMAKE_MODULE_LINKER_FLAGS="${clang_ldflags}" \
            -DCMAKE_BUILD_TYPE=Release \
            -DCMAKE_INSTALL_PREFIX="${toolchains_dir}/clang-stage4-rustcpgo" \
            -DLLVM_TARGETS_TO_BUILD=X86 \
            -DLLVM_ENABLE_PROJECTS='llvm' \
            -DLLVM_ENABLE_RUNTIMES='' \
            -DLLVM_ENABLE_LTO=Full \
            -DLLVM_BINUTILS_INCDIR=/usr/include \
            -DLLVM_INCLUDE_TESTS=NO \
            -DLLVM_INCLUDE_UTILS=YES \
            -DLLVM_INSTALL_UTILS=YES \
            -DCMAKE_BUILD_WITH_INSTALL_RPATH=YES \
            -DLLVM_PARALLEL_LINK_JOBS=3 \
            -G Ninja
        # HACK(strager): This build often OOMs when linking. On failure, build again
        # with no parallelization.
        ninja -C build-stage4-rustcpgo -k0 || :
        ninja -C build-stage4-rustcpgo -j3
        rm -rf "${toolchains_dir}/clang-stage4-rustcpgo"
        ninja -C build-stage4-rustcpgo install
    ) |& log_output build_clang_stage4_rustcpgo
}

build_rustc_stage2() {
    (
        cd "${rustc_dir}"
        rm_if_should_clean -rf build-stage2
        cat >config-stage2.toml <<EOF
changelog-seen = 2

[build]
build-dir = "${rustc_dir}/build-stage2"
cargo-native-static = true
extended = true
locked-deps = true
profiler = true
tools = ["cargo", "rustfmt", "rust-demangler"]

[rust]
channel = 'master'
codegen-units = 1
codegen-units-std = 1
debuginfo-level-std = 0
dist-src = false
jemalloc = true
lto = "fat"
parallel-compiler = true
remap-debuginfo = true

[target.x86_64-unknown-linux-gnu]
ar = "${toolchains_dir}/clang-stage2/bin/llvm-ar"
cc = "${toolchains_dir}/clang-stage2/bin/clang"
cxx = "${toolchains_dir}/clang-stage2/bin/clang++"
linker = "${toolchains_dir}/clang-stage2/bin/clang"
llvm-config = "${toolchains_dir}/clang-stage2/bin/llvm-config"
llvm-has-rust-patches = true
ranlib = "${toolchains_dir}/clang-stage2/bin/llvm-ranlib"

[install]
prefix = "${toolchains_dir}/rustc-stage2"
sysconfdir = "etc"
EOF
        rm -rf "${toolchains_dir}/rustc-stage2"
        RUSTFLAGS="-Clink-args=-fuse-ld=lld -Clink-args=-Wl,-q -Ctarget-cpu=native" \
            ./x.py --config config-stage2.toml install
    ) |& log_output build_rustc_stage2
}

build_rustc_stage3_instr() {
    (
        cd "${rustc_dir}"
        rm_if_should_clean -rf build-stage3-instr
        cat >config-stage3-instr.toml <<EOF
changelog-seen = 2

[build]
build-dir = "${rustc_dir}/build-stage3-instr"
cargo-native-static = true
extended = true
locked-deps = true
profiler = true
tools = ["cargo", "rustfmt", "rust-demangler"]
cargo = "${toolchains_dir}/rustc-stage2/bin/cargo"
rustc = "${toolchains_dir}/rustc-stage2/bin/rustc"
rustfmt = "${toolchains_dir}/rustc-stage2/bin/rustfmt"

[rust]
channel = 'master'
codegen-units = 1
codegen-units-std = 1
debuginfo-level-std = 0
dist-src = false
jemalloc = true
lto = "fat"
parallel-compiler = true
remap-debuginfo = true

[target.x86_64-unknown-linux-gnu]
ar = "${toolchains_dir}/clang-stage2/bin/llvm-ar"
cc = "${toolchains_dir}/clang-stage2/bin/clang"
cxx = "${toolchains_dir}/clang-stage2/bin/clang++"
linker = "${toolchains_dir}/clang-stage2/bin/clang"
llvm-config = "${toolchains_dir}/clang-stage3-instr/bin/llvm-config"
llvm-has-rust-patches = true
ranlib = "${toolchains_dir}/clang-stage2/bin/llvm-ranlib"

[install]
prefix = "${toolchains_dir}/rustc-stage3-instr"
sysconfdir = "etc"
EOF
        rm -rf "${toolchains_dir}/rustc-stage3-instr"
        RUSTFLAGS="-Clink-args=-fuse-ld=lld -Clink-args=-Wl,-q -Clink-args=-fprofile-generate -Ctarget-cpu=native" \
            ./x.py \
            --config config-stage3-instr.toml \
            --rust-profile-generate="${rustc_profile_dir}" \
            install
    ) |& log_output build_rustc_stage3_instr
}

build_rustc_stage4_pgo() {
    (
        cd "${rustc_dir}"
        rm_if_should_clean -rf build-stage4-pgo
        "${toolchains_dir}/clang-stage2/bin/llvm-profdata" merge --output "${rustc_profile_dir}/profdata" "${rustc_profile_dir}"/*.profraw
        cat >config-stage4-pgo.toml <<EOF
changelog-seen = 2

[build]
build-dir = "${rustc_dir}/build-stage4-pgo"
cargo-native-static = true
extended = true
locked-deps = true
profiler = true
tools = ["cargo", "rustfmt", "rust-demangler"]
cargo = "${toolchains_dir}/rustc-stage2/bin/cargo"
rustc = "${toolchains_dir}/rustc-stage2/bin/rustc"
rustfmt = "${toolchains_dir}/rustc-stage2/bin/rustfmt"

[rust]
channel = 'master'
codegen-units = 1
codegen-units-std = 1
debuginfo-level-std = 0
dist-src = false
jemalloc = true
lto = "fat"
parallel-compiler = true
remap-debuginfo = true

[target.x86_64-unknown-linux-gnu]
ar = "${toolchains_dir}/clang-stage2/bin/llvm-ar"
cc = "${toolchains_dir}/clang-stage2/bin/clang"
cxx = "${toolchains_dir}/clang-stage2/bin/clang++"
linker = "${toolchains_dir}/clang-stage2/bin/clang"
llvm-config = "${toolchains_dir}/clang-stage4-rustcpgo/bin/llvm-config"
llvm-has-rust-patches = true
ranlib = "${toolchains_dir}/clang-stage2/bin/llvm-ranlib"

[install]
prefix = "${toolchains_dir}/rustc-stage4-pgo"
sysconfdir = "etc"
EOF
        rm -rf "${toolchains_dir}/rustc-stage4-pgo"
        RUSTFLAGS="-Clink-args=-fuse-ld=lld -Clink-args=-Wl,-q -Ctarget-cpu=native" \
            ./x.py \
            --config config-stage4-pgo.toml \
            --rust-profile-use="${rustc_profile_dir}/profdata" \
            install
    ) |& log_output build_rustc_stage4_pgo
}

profile_rustc_pgo() {
    (
        rm -rf "${rustc_profile_dir}"
        PATH="${toolchains_dir}/rustc-stage3-instr/bin:${PATH}"
        if [ "$(which cargo)" != "${toolchains_dir}/rustc-stage3-instr/bin/cargo" ]; then
            echo "error: \`which cargo\` gave $(which cargo) but expected ${toolchains_dir}/rustc-stage3-instr/bin/cargo" >&2
            exit 1
        fi
        "${root}/tools/build-for-instr-rust.sh"
    ) |& log_output profile_rustc_pgo
}

build_rustc_stage4_pgo_boltinstr() {
    (
        in_dir="${toolchains_dir}/rustc-stage4-pgo"
        out_dir="${toolchains_dir}/rustc-stage4-pgo-boltinstr"

        rm -rf "${out_dir}"
        cp -a "${in_dir}" "${out_dir}"

        cd "${out_dir}"
        for exe in bin/cargo bin/rustc lib/librustc_driver-*.so lib/libstd-*.so; do
            "${toolchains_dir}/clang-stage2/bin/llvm-bolt" \
                "${in_dir}/${exe}" \
                -instrument \
                --instrumentation-file="${rustc_profile_dir}/$(basename "${exe}").bolt.fdata" \
                -o "${out_dir}/${exe}"
        done
    ) |& log_output build_rustc_stage4_pgo_boltinstr
}

profile_rustc_bolt() {
    (
        rm -f "${rustc_profile_dir}"/*.bolt.fdata
        PATH="${toolchains_dir}/rustc-stage4-pgo-boltinstr/bin:${PATH}"
        "${root}/tools/build-for-instr-rust.sh"
    ) |& log_output profile_rustc_bolt
}

build_rustc_stage4_pgo_bolt() {
    (
        in_dir="${toolchains_dir}/rustc-stage4-pgo"
        out_dir="${toolchains_dir}/rustc-stage4-pgo-bolt"

        rm -rf "${out_dir}"
        cp -a "${in_dir}" "${out_dir}"

        # HACK(strager): Delete a line from the profile data because
        # llvm-bolt barfs on it.
        sed -i -e '/_RNvXs_NtNtNtCslvDPX8Bgzk0_12rustc_middle2ty6consts3intNtB4_8ConstIntNtNtCsgch38gOM84S_4core3fmt5Debug3fmt/d' \
            "${rustc_profile_dir}"/*.bolt.fdata

        cd "${out_dir}"
        for exe in bin/cargo bin/rustc lib/librustc_driver-*.so lib/libstd-*.so; do
            "${toolchains_dir}/clang-stage2/bin/llvm-bolt" \
                "${in_dir}/${exe}" \
                -o "${out_dir}/${exe}" \
                -data="${rustc_profile_dir}/$(basename "${exe}").bolt.fdata" \
                -dyno-stats \
                -icf=1 \
                -reorder-blocks=ext-tsp \
                -reorder-functions=hfsort+ \
                -split-all-cold \
                -split-functions \
                -use-gnu-stack
        done
    ) |& log_output build_rustc_stage4_pgo_bolt
}

boltify_exe() {
    local in_exe_path="${1}"
    local out_exe_path="${2}"
    local perf_data_path="${3}"

    # https://github.com/llvm/llvm-project/blob/1b123d9fb5aaa0757b12441ae78f27ec4a25747e/bolt/docs/OptimizingClang.md
    "${toolchains_dir}/clang-stage2/bin/perf2bolt" \
        "${in_exe_path}" \
        -nl \
        -p "${perf_data_path}" \
        -o "${out_exe_path}.bolt.fdata" \
        -w "${out_exe_path}.bolt.yaml"
    "${toolchains_dir}/clang-stage2/bin/llvm-bolt" "${in_exe_path}" \
        -o "${out_exe_path}" \
        -b "${out_exe_path}.bolt.yaml" \
        -dyno-stats \
        -icf=1 \
        -reorder-blocks=ext-tsp \
        -reorder-functions=hfsort+ \
        -split-all-cold \
        -split-functions \
        -use-gnu-stack
}

log_output() {
    local name="${1}"
    mkdir -p "${logs_dir}"
    tee "${logs_dir}/${name}-$(date +%Y%m%d-%H%M%S).log"
}

rm_if_should_clean() {
    case "${clean}" in
        "") ;;
        n) ;;
        0) ;;
        false) ;;
        no) ;;
        *) rm "${@}" ;;
    esac
}

main
