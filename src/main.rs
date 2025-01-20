// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use clap::{self, command, Arg, ArgAction, ValueHint};
use cli_utils::create_output_writer;
use const_format::formatcp;
use repvar::key_value::PairBuf;
use std::collections::HashMap;
use std::env;
use std::io::Result;

mod kicad_quoter;
mod replacer;

const A_L_VERSION: &str = "version";
const A_S_VERSION: char = 'V';
const A_S_QUIET: char = 'q';
const A_L_QUIET: &str = "quiet";

fn arg_version() -> Arg {
    Arg::new(A_L_VERSION)
        .help(formatcp!(
            "Print version information and exit. \
May be combined with -{A_S_QUIET},--{A_L_QUIET}, \
to really only output the version string."
        ))
        .short(A_S_VERSION)
        .long(A_L_VERSION)
        .action(ArgAction::SetTrue)
}

fn arg_quiet() -> Arg {
    Arg::new(A_L_QUIET)
        .help("Minimize or suppress output to stdout")
        .long_help("Minimize or suppress output to stdout, and only shows log output on stderr.")
        .action(ArgAction::SetTrue)
        .short(A_S_QUIET)
        .long(A_L_QUIET)
}

fn print_version_and_exit(quiet: bool) {
    #![allow(clippy::print_stdout)]

    if !quiet {
        print!("{} ", clap::crate_name!());
    }
    println!("{}", kicad_text_injector::VERSION);
    std::process::exit(0);
}

fn main() -> Result<()> {
    let args = command!()
        .name(clap::crate_name!())
        .about("Given a KiCad PCB file (*.kicad_pcb) as input, replaces variables of the type `${KEY}` within text fields with their respective value.")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .help_expected(true)
        .disable_version_flag(true)
        .arg(arg_version())
        .arg(arg_quiet())
        .arg(
            Arg::new("input")
                .help("the input file to use; '-' for stdin")
                .num_args(0..1)
                .short('i')
                .long("input")
                .default_value("-")
                .action(ArgAction::Set)
        )
        .arg(
            Arg::new("output")
                .help("the output file to use; '-' for stdout")
                .num_args(0..1)
                .short('o')
                .long("output")
                .default_value("-")
                .action(ArgAction::Set)
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

    let quiet = args.get_flag(A_L_QUIET);
    let version = args.get_flag(A_L_VERSION);
    if version {
        print_version_and_exit(quiet);
    }

    let verbose = args.get_flag("verbose");

    let mut vars = HashMap::new();

    // enlist environment variables
    if args.get_flag("environment") {
        repvar::tools::append_env(&mut vars);
    }

    // enlist variables provided on the CLI
    if let Some(kv_pairs) = args.get_many::<PairBuf>("variable") {
        for kvp in kv_pairs {
            vars.insert(kvp.key.clone(), kvp.value.clone());
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

    let mut writer = create_output_writer(args.get_one::<String>("output"))?;

    // let settings = &repvar::settings! {vars: Box::new(vars), fail_on_missing: fail_on_missing, verbose: verbose};
    let settings = repvar::replacer::Settings::builder()
        .vars(vars)
        .fail_on_missing(fail_on_missing)
        .build();

    replacer::replace_in_stream(&settings, args.get_one("input").copied(), &mut writer)?;

    Ok(())
}
