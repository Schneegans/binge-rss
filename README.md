# BingeRSS

A minimalistice RSS reader for filtered, high-volume news feeds

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

```bash
meson setup _build/release --buildtype=release --prefix=`pwd`/_install/release
meson dist -C _build/release
flatpak-builder --force-clean --install-deps-from=flathub _repo --install build-aux/io.github.schneegans.BingeRSS.json
```