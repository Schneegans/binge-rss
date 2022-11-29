# -------------------------------------------------------------------------------------- #
#                            ___ _               ___  ___ ___                            #
#                           | _ |_)_ _  __ _ ___| _ \/ __/ __|                           #
#                           | _ \ | ' \/ _` / -_)   /\__ \__ \                           #
#                           |___/_|_||_\__, \___|_|_\|___/___/                           #
#                                      |___/                                             #
# -------------------------------------------------------------------------------------- #

# SPDX-FileCopyrightText: Simon Schneegans <code@simonschneegans.de>
# SPDX-License-Identifier: CC0-1.0

gschema_conf = configuration_data()
gschema_conf.set('app-id', app_id)
configure_file(
  input: 'BingeRSS.gschema.xml.in',
  output: '@0@.gschema.xml'.format(app_id),
  configuration: gschema_conf,
  install: true,
  install_dir: datadir / 'glib-2.0' / 'schemas'
)

metainfo_conf = configuration_data()
metainfo_conf.set('app-id', app_id)
configure_file(
  input: 'BingeRSS.metainfo.xml.in',
  output: '@0@.metainfo.xml'.format(app_id),
  configuration: metainfo_conf,
  install: true,
  install_dir: datadir / 'metainfo'
)

desktop_conf = configuration_data()
desktop_conf.set('app-id', app_id)
configure_file(
  input: 'BingeRSS.desktop.in',
  output: '@0@.desktop'.format(app_id),
  configuration: desktop_conf,
  install: true,
  install_dir: datadir / 'applications'
)

if get_option('profile') == 'develop'
  install_data(
    'icons/io.github.schneegans.BingeRSS.Devel.svg',
    install_dir: iconsdir / 'hicolor' / 'scalable' / 'apps'
  )
else
  install_data(
    'icons/io.github.schneegans.BingeRSS.svg',
    install_dir: iconsdir / 'hicolor' / 'scalable' / 'apps'
  )
endif
