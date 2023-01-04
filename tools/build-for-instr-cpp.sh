#!/usr/bin/env bash

set -e
set -u

cd "$(dirname "${0}")/../cpp"

for stdlib in libc++ libstdc++; do
  rm -rf build
  LDFLAGS=-fuse-ld=$(which mold) \
      cmake \
      -DCMAKE_C_COMPILER=clang \
      -DCMAKE_CXX_COMPILER=clang++ \
      -DCMAKE_LINKER="$(which mold)" \
      -DCMAKE_AR="$(which llvm-ar)" \
      -DCMAKE_RANLIB="$(which llvm-ranlib)" \
      -DCMAKE_BUILD_TYPE=Debug \
      -DCMAKE_CXX_FLAGS="-g0 -fpch-instantiate-templates -stdlib=${stdlib}" \
      -G Ninja -S . -B build
  ninja -C build
  for _ in {1..4}; do
      find build/{src,test}/ -name '*.cpp.o' -delete
      ninja -C build
  done
done
