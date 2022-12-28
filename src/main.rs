// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

#![warn(rust_2021_compatibility)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![warn(clippy::wildcard_enum_match_arm)]
#![warn(clippy::string_slice)]
#![warn(clippy::indexing_slicing)]
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::try_err)]
#![warn(clippy::shadow_reuse)]
#![warn(clippy::empty_structs_with_brackets)]
#![warn(clippy::else_if_without_else)]
#![warn(clippy::use_debug)]
#![warn(clippy::print_stdout)]
#![warn(clippy::print_stderr)]
// #![allow(clippy::default_trait_access)]
// NOTE allowed because:
//      If the same regex is going to be applied to multiple inputs,
//      the precomputations done by Regex construction
//      can give significantly better performance
//      than any of the `str`-based methods.
#![allow(clippy::trivial_regex)]
// #![allow(clippy::struct_excessive_bools)]
// #![allow(clippy::fn_params_excessive_bools)]

extern crate repvar;

use clap::{crate_authors, crate_version, App, Arg};
use std::collections::HashMap;
use std::env;
use std::io::Result;

mod kicad_quoter;
mod replacer;

fn main() -> Result<()> {
    let args = App::new("kicad-text-injector")
        .about("Given a KiCad PCB file (*.kicad_pcb) as input, replaces variables of the type `${KEY}` within text fields with their respective value.")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::new("input")
                .about("the input file to use; '-' for stdin")
                .takes_value(true)
                .short('i')
                .long("input")
                .multiple_occurrences(false)
                .default_value("-")
                .required(true)
        )
        .arg(
            Arg::new("output")
                .about("the output file to use; '-' for stdout")
                .takes_value(true)
                .short('o')
                .long("output")
                .multiple_occurrences(false)
                .default_value("-")
                .required(true)
        )
        .arg(
            Arg::new("variable")
                .about("a variable key-value pair to be used for substitution in the text")
                .takes_value(true)
                .short('D')
                .long("variable")
                .multiple_occurrences(true)
                .required(false)
        )
        .arg(
            Arg::new("environment")
                .about("use environment variables for substitution in the text")
                .takes_value(false)
                .short('e')
                .long("env")
                .multiple_occurrences(false)
                .required(false)
        )
        .arg(
            Arg::new("verbose")
                .about("more verbose output (useful for debugging)")
                .takes_value(false)
                .short('v')
                .long("verbose")
                .multiple_occurrences(false)
                .required(false)
        )
        .arg(
            Arg::new("fail-on-missing-values")
                .about("fail if no value is available for a variable key found in the input text")
                .takes_value(false)
                .short('f')
                .long("fail-on-missing-values")
                .multiple_occurrences(false)
                .required(false)
        )
        .get_matches();

    let verbose: bool = args.is_present("verbose");

    let mut vars = HashMap::new();

    // enlist environment variables
    if args.is_present("environment") {
        repvar::tools::append_env(&mut vars);
    }

    // enlist variables provided on the CLI
    if args.occurrences_of("variable") > 0 {
        for kvp in args
            .values_of_t::<repvar::key_value::Pair>("variable")
            .unwrap_or_else(|e| e.exit())
        {
            vars.insert(kvp.key, kvp.value);
        }
    }

    let fail_on_missing: bool = args.is_present("fail-on-missing-values");

    if verbose {
        println!();
        if let Some(in_file) = args.value_of("input") {
            println!("INPUT: {}", &in_file);
        }
        if let Some(out_file) = args.value_of("output") {
            println!("OUTPUT: {}", &out_file);
        }

        for (key, value) in &vars {
            println!("VARIABLE: {key}={value}");
        }
        println!();
    }

    let mut writer = repvar::tools::create_output_writer(args.value_of("output"))?;

    // let settings = &repvar::settings! {vars: Box::new(vars), fail_on_missing: fail_on_missing, verbose: verbose};
    let settings = repvar::replacer::Settings::builder()
        .vars(Box::new(vars))
        .fail_on_missing(fail_on_missing)
        .verbose(verbose)
        .build();

    replacer::replace_in_stream(&settings, args.value_of("input"), &mut writer)?;

    Ok(())
}
