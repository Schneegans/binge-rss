# -------------------------------------------------------------------------------------- #
#                            ___ _               ___  ___ ___                            #
#                           | _ |_)_ _  __ _ ___| _ \/ __/ __|                           #
#                           | _ \ | ' \/ _` / -_)   /\__ \__ \                           #
#                           |___/_|_||_\__, \___|_|_\|___/___/                           #
#                                      |___/                                             #
# -------------------------------------------------------------------------------------- #

# SPDX-FileCopyrightText: Simon Schneegans <code@simonschneegans.de>
# SPDX-License-Identifier: CC0-1.0

global_conf = configuration_data()
global_conf.set_quoted('app-id', app_id)
global_conf.set_quoted('version', version)
global_conf.set_quoted('profile', get_option('profile'))
config = configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: global_conf
)
run_command(
  'cp',
  meson.project_build_root() / 'src' / 'config.rs',
  meson.project_source_root() / 'src' / 'config.rs',
  check: true
)


cargo_options = [ '--manifest-path', meson.project_source_root() / 'Cargo.toml' ]
cargo_options += [ '--target-dir', meson.project_build_root() / 'src' ]

if get_option('buildtype') == 'release'
  cargo_options += [ '--release' ]
  rust_target = 'release'
  message('Building in release mode')
elif get_option('buildtype') == 'debug'
  rust_target = 'debug'
  message('Building in debug mode')
else
  error('Only the build types "release" and "debug" are supported!')
endif

cargo_env = [ 'CARGO_HOME=' + meson.project_build_root() / 'cargo-home' ]

cargo_build = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: bindir,
  command: [
    'env',
    cargo_env,
    cargo, 'build',
    cargo_options,
    '&&',
    'cp', 'src' / rust_target / meson.project_name(), '@OUTPUT@',
  ]
)
