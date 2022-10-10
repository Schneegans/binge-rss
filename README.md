# binge-rss
A minimalistice RSS reader for filtered, high-volume news feeds

## How to build

```bash
meson setup _build/release --buildtype=release --prefix=`pwd`/_install/release
cd _build/release
meson compile
meson install
```