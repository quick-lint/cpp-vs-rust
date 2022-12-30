#!/usr/bin/env bash

set -e
set -u

cd "$(dirname "${0}")/../cpp"

rm -rf build
LDFLAGS=-fuse-ld=lld \
    cmake \
    -DCMAKE_C_COMPILER=clang \
    -DCMAKE_CXX_COMPILER=clang++ \
    -DCMAKE_LINKER="$(which lld)" \
    -DCMAKE_AR="$(which llvm-ar)" \
    -DCMAKE_RANLIB="$(which llvm-ranlib)" \
    -DCMAKE_BUILD_TYPE=Debug \
    -DCMAKE_CXX_FLAGS='-g0' \
    -G Ninja -S . -B build
ninja -C build
for _ in {1..4}; do
    find build/{src,test}/ -name '*.cpp.o' -delete
    ninja -C build
done
