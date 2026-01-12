#
# Global Project Setup
#

project(
        'sys',
        default_options: [
                'rust_std=2021',
        ],
        license: 'MIT OR Apache-2.0 OR LGPL-2.1-or-later',
        meson_version: '>=1.3',
        version: '1.0.0',
)

add_languages('rust', native: false)

rust_edition = '2021'
rust_msv = '1.91'

mod_rust = import('rust')
rust = meson.get_compiler('rust')

#
# System Requirements
#

if rust.version().version_compare('<' + rust_msv)
        error('Found Rust ' + rust.version() + ' but requires >=' + rust_msv)
endif

cargo = find_program('cargo', native: true, required: true, version: '>=' + rust_msv)
jq = find_program('jq', native: true, required: true, version: '>=1.6')

#
# Import cargo metadata
#

q = run_command(
        [
                './lib/util/cargo-query.sh',

                # The script does not use any of these arguments, but meson
                # thinks it does, and triggers a reconfiguration if they
                # change. We use this neat trick to make meson depend on those
                # files.
                './Cargo.toml',
                './lib/osi/src/Cargo.toml',
                './lib/sys/src/Cargo.toml',
                './lib/tmp/src/Cargo.toml',
        ],
        capture: true,
        check: true,
        env: {
                'BIN_CARGO': cargo.full_path(),
                'BIN_JQ': jq.full_path(),
                'PATH_CARGO_TOML': meson.current_source_dir() / 'Cargo.toml',
        },
).stdout().splitlines()

libosi_version = q[0]
libsys_version = q[1]
libtmp_version = q[2]

libosi_major = libosi_version.split('.')[0]
libsys_major = libsys_version.split('.')[0]
libtmp_major = libtmp_version.split('.')[0]

summary({
        'version osi': libosi_version,
        'version sys': libsys_version,
        'version tmp': libtmp_version,
})

#
# Config: doctest
#

use_doctest = get_option('doctest')

#
# Config: libc
#

use_libc = get_option('libc')
if use_libc
        dep_libc = dependency('libc-rs-0.2', version: '>=0.2.175')
endif

#
# Config: std
#

use_std = get_option('std')

#
# Target: libosi
#

libosi_rlib_rust_args = []

if use_std
        libosi_rlib_rust_args += ['--cfg=feature="std"']
endif

libosi_rlib = static_library(
        'osi',
        ['lib/osi/src/lib.rs'],
        rust_args: libosi_rlib_rust_args,
)

libosi_dep = declare_dependency(
        link_with: libosi_rlib,
        version: libosi_version,
)

meson.override_dependency('libosi-' + libosi_major, libosi_dep, static: true)

#
# Target: libsys
#

libsys_rlib_deps = [libosi_dep]
libsys_rlib_rust_args = []

if use_libc
        libsys_rlib_deps += [dep_libc]
        libsys_rlib_rust_args += ['--cfg=feature="libc"']
endif

if use_std
        libsys_rlib_rust_args += ['--cfg=feature="std"']
endif

libsys_rlib = static_library(
        'sys',
        ['lib/sys/src/lib.rs'],
        dependencies: libsys_rlib_deps,
        rust_args: libsys_rlib_rust_args,
)

libsys_dep = declare_dependency(
        dependencies: [libosi_dep],
        link_with: libsys_rlib,
        version: libsys_version,
)

meson.override_dependency('libsys-' + libsys_major, libsys_dep, static: true)

#
# Target: libtmp
#

libtmp_rlib = static_library(
        'tmp',
        ['lib/tmp/src/lib.rs'],
        dependencies: [
                libosi_dep,
                libsys_dep,
        ],
)

libtmp_dep = declare_dependency(
        dependencies: [
                libosi_dep,
                libsys_dep,
        ],
        link_with: libtmp_rlib,
        version: libtmp_version,
)

meson.override_dependency('libtmp-' + libtmp_major, libtmp_dep, static: true)

#
# Target: test-*
#

mod_rust.test(
        'osi-test',
        libosi_rlib,
        suite: 'unit',
)

mod_rust.test(
        'sys-test',
        libsys_rlib,
        suite: 'unit',
)

mod_rust.test(
        'tmp-test',
        libtmp_rlib,
        suite: 'unit',
)

if use_doctest and meson.version().version_compare('>=1.8')
        mod_rust.doctest(
                'osi-doctest',
                libosi_rlib,
                suite: 'doctest',
        )
        mod_rust.doctest(
                'sys-doctest',
                libsys_rlib,
                suite: 'doctest',
        )
        mod_rust.doctest(
                'tmp-doctest',
                libtmp_rlib,
                suite: 'doctest',
        )
endif
