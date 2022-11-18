<!--
SPDX-FileCopyrightText: Simon Schneegans <code@simonschneegans.de>
SPDX-License-Identifier: CC-BY-4.0
-->

# BingeRSS

A mimalistic RSS reader for fast, filtered, high-volume news feeds.

## Local Installation

### Building

```bash
meson setup _build/release --buildtype=release --prefix=`pwd`/_install/release
meson install -C _build/release
```

### Running

```bash
XDG_DATA_DIRS=$XDG_DATA_DIRS:`pwd`/_install/release/share ./_install/release/bin/binge-rss
```

## Flatpak Installation

### Building & Installing

```bash
meson setup _build/release --buildtype=release --prefix=`pwd`/_install/release
meson dist -C _build/release
flatpak-builder --user --install --force-clean --install-deps-from=flathub _repo tools/io.github.schneegans.BingeRSS.json
```

### Running

```bash
flatpak run io.github.schneegans.BingeRSS
```

## Debug Configuration

Write this to GSettings key `/io/github/schneegans/BingeRSS/feeds`.

```json
[{"title":"Der SPIEGEL","url":"https://www.spiegel.de/schlagzeilen/tops/index.rss","viewed":"2022-10-09 16:06:14 UTC","filter":[]},{"title":"Unixporn","url":"http://reddit.com/r/unixporn/new/.rss?sort=new","viewed":"2022-10-09 16:06:14 UTC","filter":[]},{"title":"Forschung Aktuell","url":"https://www.deutschlandfunk.de/forschung-aktuell-104.xml","viewed":"2022-10-09 16:06:14 UTC","filter":[]},{"title":"Linux","url":"http://reddit.com/r/linux/new/.rss?sort=new","viewed":"2022-10-09 16:06:14 UTC","filter":[]},{"title":"GNOME","url":"http://reddit.com/r/gnome/new/.rss?sort=new","viewed":"2022-10-09 16:06:14 UTC","filter":[]},{"title":"OMG Ubuntu","url":"https://omgubuntu.co.uk/feed","viewed":"2022-10-09 16:06:14 UTC","filter":[]},{"title":"Blendernation","url":"https://www.blendernation.com/feed/","viewed":"2022-10-09 16:06:14 UTC","filter":[]},{"title":"The Verge","url":"https://www.theverge.com/rss/index.xml","viewed":"2022-10-09 16:06:14 UTC","filter":[]},{"title":"Ars Technica","url":"https://feeds.arstechnica.com/arstechnica/features","viewed":"2022-10-09 16:06:14 UTC","filter":[]},{"title":"Hacker News","url":"https://news.ycombinator.com/rss","viewed":"2022-10-09 16:06:14 UTC","filter":[]},{"title":"Vulnerabilities","url":"https://nvd.nist.gov/feeds/xml/cve/misc/nvd-rss-analyzed.xml","viewed":"2022-10-09 16:06:14 UTC","filter":[]}]
```