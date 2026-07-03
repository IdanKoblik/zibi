#!/bin/bash

set -e

rm -rf Zibi.AppDir
mkdir -p Zibi.AppDir/usr/bin/

cargo build --release
pyinstaller --onefile \
    --collect-all mediapipe \
    --add-data models:models \
    track.py

cp target/release/zibi \
   Zibi.AppDir/usr/bin/

cp dist/track \
   Zibi.AppDir/usr/bin/

cp packaging/AppRun Zibi.AppDir/
cp assets/zibi.desktop Zibi.AppDir/
cp assets/zibi.png Zibi.AppDir/

ARCH=x86_64 appimagetool Zibi.AppDir
