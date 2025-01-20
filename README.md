# KiCad text injector

<!--
SPDX-FileCopyrightText: 2021-2023 Robin Vobruba <hoijui.quaero@gmail.com>

SPDX-License-Identifier: CC0-1.0
-->

[![License: GPL-3.0-or-later](
    https://img.shields.io/badge/License-GPL%203.0+-blue.svg)](
    https://www.gnu.org/licenses/gpl-3.0.html)
[![REUSE status](
    https://api.reuse.software/badge/github.com/hoijui/kicad-text-injector)](
    https://api.reuse.software/info/github.com/hoijui/kicad-text-injector)
[![crates.io](
    https://img.shields.io/crates/v/kicad-text-injector.svg)](
    https://crates.io/crates/kicad-text-injector)
[![Docs](
    https://docs.rs/kicad-text-injector/badge.svg)](
    https://docs.rs/kicad-text-injector)
[![dependency status](
    https://deps.rs/repo/github/hoijui/kicad-text-injector/status.svg)](
    https://deps.rs/repo/github/hoijui/kicad-text-injector)
[![Build status](
    https://github.com/hoijui/kicad-text-injector/workflows/build/badge.svg)](
    https://github.com/hoijui/kicad-text-injector/actions)

[![In cooperation with FabCity Hamburg](
    https://raw.githubusercontent.com/osegermany/tiny-files/master/res/media/img/badge-fchh.svg)](
    https://fabcity.hamburg)
[![In cooperation with Open Source Ecology Germany](
    https://raw.githubusercontent.com/osegermany/tiny-files/master/res/media/img/badge-oseg.svg)](
    https://opensourceecology.de)

This tool allows you to post-process your KiCad PCB files,
by replacing variables of the type `${NAME}` in your text elements.

You may put placeholder text onto your PCB -
for example `${PROJECT_REPO_URL}` -
on any layer, and this tool then fills in the actual value,
for example `https://github.com/myorg/myproj`.
This is most useful for filling in project-specific meta-data into the final output,
and thus this tool is primarily targeting CI jobs,
though it can also be run locally.

## How to compile

You need to install Rust(lang) and Cargo.

Then you can run:

```bash
scripts/build
```

## Get the tool

As for now, you have two choices:

1. [Compile it](#how-to-compile) yourself
1. Download the Linux x86\_64 statically linked binary from
   [the releases page](https://github.com/hoijui/kicad-text-injector/releases)

## Usage

```text
$ kicad-text-injector --help
Given a KiCad PCB file (*.kicad_pcb) as input, replaces variables of the type `${KEY}` within text
fields with their respective value.

USAGE:
    kicad-text-injector [FLAGS] [OPTIONS] --input <input> --output <output>

FLAGS:
    -e, --env                       use environment variables for substitution in the text
    -f, --fail-on-missing-values    fail if no value is available for a variable key found in the
                                    input text
    -h, --help                      Prints help information
    -v, --verbose                   more verbose output (useful for debugging)
    -V, --version                   Prints version information

OPTIONS:
    -i, --input <input>             the input file to use; '-' for stdin [default: -]
    -o, --output <output>           the output file to use; '-' for stdout [default: -]
    -D, --variable <variable>...    a variable key-value pair to be used for substitution in the
                                    text
```

## Misc

We very warmly recommend you to use
**the [KiBot](https://github.com/INTI-CMNB/KiBot) tool**
for the actual generation of the final output
from the post-processed KiCad sources.
It can generate much more then just Gerbers
and 2D renders of the PCBs.

Also see the [KiCad image/QRCode injector](
https://github.com/hoijui/kicad-image-injector).

## Funding

This project was funded by the European Regional Development Fund (ERDF)
in the context of the [INTERFACER Project](https://www.interfacerproject.eu/),
from July 2021
until March 2023.

![Logo of the EU ERDF program](
    https://cloud.fabcity.hamburg/s/TopenKEHkWJ8j5P/download/logo-eu-erdf.png)
