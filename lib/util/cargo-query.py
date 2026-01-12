#!/usr/bin/env python3
#
# Parse `Cargo.toml` of the workspace and each package. Extract all data
# relevant to the Meson integration.
#
# Data is printed on `stdout`, for easy consumption in Meson. Note that
# Meson still lacks any parser for structured input like JSON, so instead
# newline and comma separated values are used. Since input is trusted and
# contains no comma or newline, this is safe (it would still be much nicer
# if we could use JSON).
#

import os
import tomllib

def toml_lints(toml):
    groups = ['clippy', 'rust']
    acc_allow = ''
    for g in groups:
        for k in toml_ws['workspace']['lints'][g]:
            v = toml_ws['workspace']['lints'][g][k]
            level = v['level'] if isinstance(v, dict) else v
            if level == 'allow':
                acc_allow += ',' if acc_allow else ''
                acc_allow += 'clippy::' + k
    return acc_allow

def toml_version(toml):
    return toml['package']['version']

if __name__ == "__main__":
    path_ws = os.environ.get('PATH_WS', '.')

    path_cargo_ws = os.path.join(path_ws, 'Cargo.toml')
    path_cargo_osi = os.path.join(path_ws, 'lib/osi/Cargo.toml')
    path_cargo_sys = os.path.join(path_ws, 'lib/sys/Cargo.toml')
    path_cargo_tmp = os.path.join(path_ws, 'lib/tmp/Cargo.toml')

    with open(path_cargo_ws, mode="rb") as f:
        toml_ws = tomllib.load(f)
    with open(path_cargo_osi, mode="rb") as f:
        toml_osi = tomllib.load(f)
    with open(path_cargo_sys, mode="rb") as f:
        toml_sys = tomllib.load(f)
    with open(path_cargo_tmp, mode="rb") as f:
        toml_tmp = tomllib.load(f)

    print(toml_version(toml_osi))
    print(toml_version(toml_sys))
    print(toml_version(toml_tmp))
    print(toml_lints(toml_ws))
