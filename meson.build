# -------------------------------------------------------------------------------------- #
#                            ___ _               ___  ___ ___                            #
#                           | _ |_)_ _  __ _ ___| _ \/ __/ __|                           #
#                           | _ \ | ' \/ _` / -_)   /\__ \__ \                           #
#                           |___/_|_||_\__, \___|_|_\|___/___/                           #
#                                      |___/                                             #
# -------------------------------------------------------------------------------------- #

# SPDX-FileCopyrightText: Simon Schneegans <code@simonschneegans.de>
# SPDX-License-Identifier: CC0-1.0

project(
  'binge-rss',
  'rust',
  version: '0.1.0',
  meson_version: '>= 0.59',
  license: 'MIT',
)

gnome = import('gnome')

app_id = 'io.github.schneegans.BingeRSS'
version = meson.project_version()

if get_option('profile') == 'develop'
  app_id = 'io.github.schneegans.BingeRSS.Devel'

  vcs_tag = run_command('git', 'rev-parse', '--short', 'HEAD', check: false).stdout().strip()
  if vcs_tag == ''
    version = '@0@.devel'.format(version)
  else
    version = '@0@.@1@'.format(version, vcs_tag)
  endif

endif

dependency('glib-2.0', version: '>= 2.66')
dependency('gio-2.0', version: '>= 2.66')
dependency('gtk4', version: '>= 4.6.0')
dependency('libadwaita-1', version: '>= 1.1.0')

glib_compile_schemas = find_program('glib-compile-schemas', required: true)
cargo = find_program('cargo', required: true)


prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
datadir = prefix / get_option('datadir')
iconsdir = datadir / 'icons'
# pkgdatadir = datadir / meson.project_name()

meson.add_dist_script(
  'tools/vendor.sh',
  meson.project_build_root() / 'meson-dist' / meson.project_name() + '-' + version,
  meson.project_source_root()
)


subdir('data')
subdir('src')

gnome.post_install(
  gtk_update_icon_cache: true,
  glib_compile_schemas: true,
  update_desktop_database: true,
)
