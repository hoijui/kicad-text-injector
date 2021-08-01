<!--
SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>

SPDX-License-Identifier: CC0-1.0
-->

# KiCad text injector

[![License: GPL-3.0-or-later](
https://img.shields.io/badge/License-GPL%203.0+-blue.svg)](
https://www.gnu.org/licenses/gpl-3.0.html)
[![REUSE status](
https://api.reuse.software/badge/github.com/hoijui/kicad-text-injector)](
https://api.reuse.software/info/github.com/hoijui/kicad-text-injector)

This tool allows you to post-process your KiCad PCB files,
by replacing variables of the type `${NAME}` in your text elements.

You may put placeholder text onto your PCB -
for example `${PROJECT_REPO_URL}` -
on any layer, and this tool then fills in the actual value,
for example `https://github.com/myorg/myproj`.
This is most useful for filling in project-specific meta-data into the final output,
and thus this tool is primarily targeting CI jobs,
though it can also be run locally.

## Dependencies

* Python 3
* BASH

## How it works

### Install Prerequisites

* BASH
* git
* Python
* `pcb-tools` (a Python library)

on a regular Debian based Linux,
you can install all of this with:

```bash
sudo apt-get install bash git python3-pip
pip install -r requirements.txt
```

### Get the tool

In the repo of your project in which you want to use this tool,
which would be one that supports *\*.kicad_pcb* files,
you would do the following to install this tool (in the project root dir):

```bash
mkdir -p doc-tools
git submodule add https://github.com/hoijui/kicad-text-injector.git doc-tools/kicad-text-injector
pip install -r doc-tools/kicad-text-injector/requirements.txt
```

> **NOTE**\
> There might be a tool to automate this in a more user friendly way,
> comparable to a package manager like `Oh-My-ZSH` or `apt`.

### Run

This will generate the PCB derived artifacts for all KiCad PCBs in the repo:

```bash
doc-tools/kicad-text-injector/generate_sources
```

Output can be found under the *build* directory.

## Misc

We very warmly recommend you to use
**the [KiBot](https://github.com/INTI-CMNB/KiBot) tool**
for the actual generation of the final output
from the post-processed KiCad sources.
It can generate much more then just Gerbers
and 2D renders of the PCBs.

Also see the [KiCad image/QRCode injector](
https://github.com/hoijui/kicad-image-injector).

