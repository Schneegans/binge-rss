project(
  'binge-rss',
  'rust',
  version: '0.1.0',
  meson_version: '>= 0.60',
  license: 'MIT',
)

gnome = import('gnome')

base_id = 'io.github.schneegans.BingeRSS'

dependency('glib-2.0', version: '>= 2.66')
dependency('gio-2.0', version: '>= 2.66')
dependency('gtk4', version: '>= 4.6.0')
dependency('libadwaita-1', version: '>= 1.1.0')

# glib_compile_resources = find_program('glib-compile-resources', required: true)
glib_compile_schemas = find_program('glib-compile-schemas', required: true)
cargo = find_program('cargo', required: true)

version = meson.project_version()

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
datadir = prefix / get_option('datadir')
iconsdir = datadir / 'icons'
# pkgdatadir = datadir / meson.project_name()

subdir('data')
subdir('src')

gnome.post_install(
  gtk_update_icon_cache: true,
  glib_compile_schemas: true,
  update_desktop_database: true,
)