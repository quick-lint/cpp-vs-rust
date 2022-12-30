#!/usr/bin/env bash
set -e
set -u

cd "$(dirname "${0}")/.."

build_cpp_with_clang() {
    (
        PATH="${HOME}/Toolchains/clang-stage2/bin/clang:${PATH}"
        cmake \
            -DCMAKE_C_COMPILER=clang \
            -DCMAKE_CXX_COMPILER=clang++ \
            -DCMAKE_CXX_FLAGS='-march=native -mtune=native -fPIC' \
            -DCMAKE_BUILD_TYPE=Release \
            -DCMAKE_INTERPROCEDURAL_OPTIMIZATION=TRUE \
            -G Ninja -S cpp -B cpp/build-perf-clang
        ninja -C cpp/build-perf-clang quick-lint-js-c-api
    )
}

build_cpp_with_gcc() {
    cmake \
        -DCMAKE_C_COMPILER=gcc-12 \
        -DCMAKE_CXX_COMPILER=g++-12 \
        -DCMAKE_CXX_FLAGS='-march=native -mtune=native -fPIC' \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_INTERPROCEDURAL_OPTIMIZATION=TRUE \
        -G Ninja -S cpp -B cpp/build-perf-gcc
    ninja -C cpp/build-perf-gcc quick-lint-js-c-api
}

build_rust() {
    (
        cd rust
        RUSTFLAGS='-Ctarget-cpu=native' cargo build --release --lib
    )
}

build_cpp_with_clang
build_cpp_with_gcc
build_rust

mkdir -p build-perf
cp cpp/build-perf-clang/src/libquick-lint-js-c-api.so build-perf/a.so
cp cpp/build-perf-gcc/src/libquick-lint-js-c-api.so build-perf/b.so
cp rust/target/release/libcpp_vs_rust.so build-perf/c.so
for so in a b c; do
    strip -o "build-perf/${so}s.so" "build-perf/${so}.so"
done

cmake \
    -DCMAKE_CXX_FLAGS='-march=native -mtune=native' \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_INTERPROCEDURAL_OPTIMIZATION=TRUE \
    -G Ninja -S benchmark -B benchmark/build
ninja -C benchmark/build cpp-vs-rust-benchmark

for so in a b c; do
    #perf record -o build-perf/perf-${so}.data ...
    CPP_VS_RUST_DLL="build-perf/${so}.so" \
        ./benchmark/build/cpp-vs-rust-benchmark \
        --benchmark_out="build-perf/${so}-results.json" \
        --benchmark_out_format=json \
        --benchmark_min_time=0.5 \
        --benchmark_repetitions=10
done
