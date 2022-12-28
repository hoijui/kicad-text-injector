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

use clap::{self, command, Arg, ArgAction, ValueHint};
use repvar::key_value::PairBuf;
use std::collections::HashMap;
use std::env;
use std::io::Result;

mod kicad_quoter;
mod replacer;

fn main() -> Result<()> {
    let args = command!()
        .name(clap::crate_name!())
        .about("Given a KiCad PCB file (*.kicad_pcb) as input, replaces variables of the type `${KEY}` within text fields with their respective value.")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .arg(
            Arg::new("input")
                .help("the input file to use; '-' for stdin")
                .num_args(0..1)
                .short('i')
                .long("input")
                .default_value("-")
                .action(ArgAction::Set)
                .required(true)
        )
        .arg(
            Arg::new("output")
                .help("the output file to use; '-' for stdout")
                .num_args(0..1)
                .short('o')
                .long("output")
                .default_value("-")
                .action(ArgAction::Set)
                .required(true)
        )
        .arg(
            Arg::new("variable")
                .help("a variable key-value pair to be used for substitution in the text")
                .short('D')
                .long("variable")
                .value_parser(PairBuf::parse)
                .value_hint(ValueHint::Other)
                .value_name("KEY=VALUE")
                .action(ArgAction::Append)
        )
        .arg(
            Arg::new("environment")
                .help("use environment variables for substitution in the text")
                .short('e')
                .long("env")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("verbose")
                .help("more verbose output (useful for debugging)")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("fail-on-missing-values")
                .help("fail if no value is available for a variable key found in the input text")
                .short('f')
                .long("fail-on-missing-values")
                .action(ArgAction::SetTrue)
        )
        .get_matches();

    let verbose = args.get_flag("verbose");

    let mut vars = HashMap::new();

    // enlist environment variables
    if args.get_flag("environment") {
        repvar::tools::append_env(&mut vars);
    }

    // enlist variables provided on the CLI
    if let Some(kvps) = args.get_many::<PairBuf>("variable") {
        for kvp in kvps {
            vars.insert(kvp.key.to_owned(), kvp.value.to_owned());
        }
    }

    let fail_on_missing = args.get_flag("fail-on-missing-values");

    if verbose {
        println!();
        if let Some(in_file) = args.get_one::<String>("input") {
            println!("INPUT: {}", &in_file);
        }
        if let Some(out_file) = args.get_one::<String>("output") {
            println!("OUTPUT: {}", &out_file);
        }

        for (key, value) in &vars {
            println!("VARIABLE: {key}={value}");
        }
        println!();
    }

    let mut writer = repvar::tools::create_output_writer(args.get_one("output").copied())?;

    // let settings = &repvar::settings! {vars: Box::new(vars), fail_on_missing: fail_on_missing, verbose: verbose};
    let settings = repvar::replacer::Settings::builder()
        .vars(vars)
        .fail_on_missing(fail_on_missing)
        .verbose(verbose)
        .build();

    replacer::replace_in_stream(&settings, args.get_one("input").copied(), &mut writer)?;

    Ok(())
}
