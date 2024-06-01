// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod kicad_quoter;
pub mod replacer;

use git_version::git_version;

pub const VERSION: &str = git_version!();
