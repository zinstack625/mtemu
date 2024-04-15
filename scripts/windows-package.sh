#!/usr/bin/env sh

###
# This is meant to be run from under MSYS2 Clang64 context
#
# Make sure that dependencies are already installed. This does not
# make any effort to ensure these dependencies are present
###

PREFIX="Program Files (x86)/MTemu"

meson setup --prefix "C:/$PREFIX" --buildtype release
meson install -C build --destdir dest || true
cp build/src/mtemu.exe build/dest/$PREFIX/
ldd build/src/mtemu.exe | grep "\/clang64.*\.dll" | awk '{print $3;}' | xargs -I{} cp {} "build/dest/$PREFIX/"
cp "build/dest/$PREFIX/bin/libengine.dll" "build/dest/$PREFIX"
rm -r "build/dest/$PREFIX/bin" "build/dest/$PREFIX/lib"
