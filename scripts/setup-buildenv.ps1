# Install Chocolatey (windows package manager, sort of like brew --cask)
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

$scriptFile = New-TemporaryFile
@'
#!/usr/bin/env sh
PACKAGES="mingw-w64-clang-x86_64-toolchain \
    mingw-w64-clang-x86_64-meson \
    mingw-w64-clang-x86_64-rust \
    mingw-w64-clang-x86_64-gtk4 \
    mingw-w64-clang-x86_64-mono \
    mingw-w64-clang-x86_64-libadwaita"
 
pacman -Sy $PACKAGES
'@ | Out-File -Encoding utf8 $scriptFile

choco install msys2
C:\tools\msys64\clang64.exe $scriptFile