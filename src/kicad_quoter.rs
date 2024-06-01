// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;

#[must_use]
pub fn quote(input: &str) -> Cow<'_, str> {
    lazy_static! {
        static ref PCBNEW_TEXT_QUOTE: Regex = Regex::new(
            r#"(?P<pre>\((gr_text|fp_text\s+[a-z]+)\s+)(?P<text>[^"\s]*\$\{[-_0-9a-zA-Z]*\}[^\s"]*)"#
            // r#"(?P<pre>\((gr_text|fp_text\s+[a-z]+)\s+)(?P<text>[^"\s]*\$\{[-_0-9a-zA-Z]*\}[^\s"]*)(?P<post>\s+[\)\(])"#
        ).unwrap();
    }
    PCBNEW_TEXT_QUOTE.replace_all(input, r#"$pre"$text""#)
    // PCBNEW_TEXT_QUOTE.replace_all(input, r#"$pre"$text"$post"#)
}

#[must_use]
pub fn unquote(input: &str) -> Cow<'_, str> {
    lazy_static! {
        static ref PCBNEW_TEXT_UNQUOTE: Regex = Regex::new(
            r#"(?P<pre>\((gr_text|fp_text\s+[a-z]+)\s+)"(?P<text>[^"\s\\]+)"(?P<post>\s+[\)\(])"#
        )
        .unwrap();
    }
    PCBNEW_TEXT_UNQUOTE.replace_all(input, r"$pre$text$post")
}

#[cfg(test)]
mod tests {
    // Note this useful idiom:
    // importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_quote_not_no_var_fp() {
        let input = r#"    (fp_text user %R (at 0.3 0) (layer F.Fab)\n"#;
        let expected = input;
        let actual = quote(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_quote_not_no_var_gr() {
        let input = r#"  (gr_text Batch: (at 146.558 135.636) (layer B.SilkS) (tstamp 5F4CC676)\n"#;
        let expected = input;
        let actual = quote(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_quote_not_quoted_no_var_fp() {
        let input = r#"    (fp_text user "%R 2" (at 0.3 0) (layer F.Fab)\n"#;
        let expected = input;
        let actual = quote(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_quote_not_quoted_no_var_gr() {
        let input =
            r#"  (gr_text "Batch: abc" (at 146.558 135.636) (layer B.SilkS) (tstamp 5F4CC676)\n"#;
        let expected = input;
        let actual = quote(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_quote_var_fp() {
        let input = r#"    (fp_text user ${PROJECT_BATCH_ID} (at 0.3 0) (layer F.Fab)\n"#;
        let expected = r#"    (fp_text user "${PROJECT_BATCH_ID}" (at 0.3 0) (layer F.Fab)\n"#;
        let actual = quote(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_quote_var_gr() {
        let input = r#"  (gr_text ${PROJECT_BATCH_ID} (at 145.288 135.636) (layer B.SilkS) (tstamp 5F4CC67E)\n"#;
        let expected = r#"  (gr_text "${PROJECT_BATCH_ID}" (at 145.288 135.636) (layer B.SilkS) (tstamp 5F4CC67E)\n"#;
        let actual = quote(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_quote_not_quoted_var_fp() {
        let input = r#"    (fp_text user "Batch: ${PROJECT_BATCH_ID}" (at 0.3 0) (layer F.Fab)\n"#;
        let expected = input;
        let actual = quote(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_quote_not_quoted_var_gr() {
        let input = r#"  (gr_text "Batch: ${PROJECT_BATCH_ID}" (at 145.288 135.636) (layer B.SilkS) (tstamp 5F4CC67E)\n"#;
        let expected = input;
        let actual = quote(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_unquote_not_unquoted_fp() {
        let input = r#"    (fp_text user %R (at 0.3 0) (layer F.Fab)\n"#;
        let expected = input;
        let actual = unquote(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_unquote_not_unquoted_gr() {
        let input = r#"  (gr_text Batch: (at 146.558 135.636) (layer B.SilkS) (tstamp 5F4CC676)\n"#;
        let expected = input;
        let actual = unquote(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_unquote_not_with_space_fp() {
        let input = r#"    (fp_text user "%R 2" (at 0.3 0) (layer F.Fab)\n"#;
        let expected = input;
        let actual = unquote(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_unquote_not_with_space_gr() {
        let input =
            r#"  (gr_text "Batch: abc" (at 146.558 135.636) (layer B.SilkS) (tstamp 5F4CC676)\n"#;
        let expected = input;
        let actual = unquote(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_unquote_no_var_fp() {
        let input = r#"    (fp_text user "%R" (at 0.3 0) (layer F.Fab)\n"#;
        let expected = r#"    (fp_text user %R (at 0.3 0) (layer F.Fab)\n"#;
        let actual = unquote(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_unquote_no_var_gr() {
        let input =
            r#"  (gr_text "Batch:" (at 146.558 135.636) (layer B.SilkS) (tstamp 5F4CC676)\n"#;
        let expected =
            r#"  (gr_text Batch: (at 146.558 135.636) (layer B.SilkS) (tstamp 5F4CC676)\n"#;
        let actual = unquote(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_unquote_var_fp() {
        let input = r#"    (fp_text user "${PROJECT_BATCH_ID}" (at 0.3 0) (layer F.Fab)\n"#;
        let expected = r#"    (fp_text user ${PROJECT_BATCH_ID} (at 0.3 0) (layer F.Fab)\n"#;
        let actual = unquote(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_unquote_var_gr() {
        let input = r#"  (gr_text "${PROJECT_BATCH_ID}" (at 145.288 135.636) (layer B.SilkS) (tstamp 5F4CC67E)\n"#;
        let expected = r#"  (gr_text ${PROJECT_BATCH_ID} (at 145.288 135.636) (layer B.SilkS) (tstamp 5F4CC67E)\n"#;
        let actual = unquote(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_unquote_not_var_with_space_fp() {
        let input = r#"    (fp_text user "Batch: ${PROJECT_BATCH_ID}" (at 0.3 0) (layer F.Fab)\n"#;
        let expected = input;
        let actual = unquote(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_unquote_not_var_with_space_gr() {
        let input = r#"  (gr_text "Batch: ${PROJECT_BATCH_ID}" (at 145.288 135.636) (layer B.SilkS) (tstamp 5F4CC67E)\n"#;
        let expected = input;
        let actual = unquote(input);
        assert_eq!(expected, actual);
    }
}
