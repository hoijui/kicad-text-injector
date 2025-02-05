// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io;
use std::io::BufRead;
use std::io::Write;
use std::path::Path;

use cli_utils::create_input_reader;

use crate::kicad_quoter;

/// Replaces all occurrences of variables of the form `${KEY}` -
/// in a KiCad file supplied as an input stream -
/// with their respective values.
///
/// # Errors
///
/// If a variable key was found in the stream,
/// but `vars` contains no entry for it,
/// and `fail_on_missing` is `true`.
///
/// If reading from the input failed.
///
/// If writing to the output failed.
pub fn replace_in_stream<P: AsRef<Path>, S: ::std::hash::BuildHasher>(
    settings: &repvar::replacer::Settings<S>,
    input: Option<P>,
    writer: &mut Box<dyn Write>,
) -> io::Result<()> {
    let reader = create_input_reader(input)?;

    reader
        .lines()
        .map(|line_res| -> io::Result<()> {
            match line_res {
                Ok(line) => {
                    let quoted = kicad_quoter::quote(&line);
                    let replaced = repvar::replacer::replace_in_string(&quoted, settings)?;
                    let unquoted = kicad_quoter::unquote(replaced.as_ref());
                    writer.write_all(unquoted.as_bytes())?;
                    writer.write_all(b"\n")?;
                    Ok(())
                }
                Err(err) => Err(err),
            }
        })
        .try_for_each(|err| -> io::Result<()> { err })?;

    Ok(())
}
