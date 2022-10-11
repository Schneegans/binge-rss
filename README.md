<!--
SPDX-FileCopyrightText: Simon Schneegans <code@simonschneegans.de>
SPDX-License-Identifier: CC-BY-4.0
-->

# BingeRSS

A mimalistic RSS reader for fast, filtered, high-volume news feeds.

## Local

### Building

```bash
meson setup _build/release --buildtype=release --prefix=`pwd`/_install/release
meson install -C _build/release
```

### Running

```bash
XDG_DATA_DIRS=$XDG_DATA_DIRS:`pwd`/_install/release/share ./_install/release/bin/binge-rss
```

## Flatpak

### Building & Installing

```bash
meson setup _build/release --buildtype=release --prefix=`pwd`/_install/release
meson dist -C _build/release
flatpak-builder --user --install --force-clean --install-deps-from=flathub _repo build-aux/io.github.schneegans.BingeRSS.json
```

### Running

```bash
flatpak run io.github.schneegans.BingeRSS
```