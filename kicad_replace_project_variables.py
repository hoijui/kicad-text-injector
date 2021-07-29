# SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
#
# SPDX-License-Identifier: GPL-3.0-or-later

'''
Allows to replace variables of the form `${KEY}` with actual values
within text fields on KiCad PCB (pcbnew) boards.
This is most useful for injecting build-specific values when generating output
(renders, gerbers, ...) in CI builds.
'''

import re
import os
import sys
from datetime import date
from pathlib import Path

import click
from git import Repo
import pcbnew

import replace_vars

CONTEXT_SETTINGS = dict(help_option_names=['-h', '--help'])
DATE_FORMAT="%Y-%m-%d"
R_DIRTY_VERSION = re.compile(".*-dirty(-.*)?$")

# Quotes all KiCad text entries that are not yet quoted and contain a variable of the form '${KEY}'
filter_kicad_quote   = replace_vars.RegexTextFilter(
        r'(?P<pre>\(gr_text\s+)(?P<text>[^"\s]*\${[-_0-9a-zA-Z]*}[^\s"]*)(?P<post>\s+[\)\(])',
        r'\g<pre>"\g<text>"\g<post>')
# Unquotes all KiCad text entries that are quoted and do not contain white-space
filter_kicad_unquote = replace_vars.RegexTextFilter(
        r'(?P<pre>\(gr_text\s+)"(?P<text>[^"\s\\]+)"(?P<post>\s+[\)\(])',
        r'\g<pre>\g<text>\g<post>')

@click.group(context_settings=CONTEXT_SETTINGS)
@click.version_option()
def kicad_replace_project_vars() -> None:
    '''
    Stub function for module-global click annotations.
    '''

def git_remote_to_https_url(url) -> str:
    '''
    Converts a common git remote URL
    into a web-ready (http(s)) URL of the project.

    for example:

    "git@github.com:hoijui/kicad-text-injector.git"
    ->
    "https://github.com/hoijui/kicad-text-injector"
    '''
    public_url = re.sub(r"^git@", "https://", url)
    public_url = public_url.replace(".com:", ".com/", 1)
    public_url = re.sub(r"\.git$", "", public_url)
    return public_url

def is_dirty_version(vers) -> bool:
    '''
    Checks whether a given version string is a git dirty version.
    Dirty means, there are uncommitted changes.
    '''
    return R_DIRTY_VERSION.match(vers) # TODO check if correct

def test_is_dirty_version():
    '''
    Unit test for the function is_dirty_version.
    '''
    assert not is_dirty_version('my_proj-1.2.3')
    assert not is_dirty_version('my_proj-1.2.3dirty')
    assert not is_dirty_version('my_proj-1.2.3-dirtybroken')
    assert is_dirty_version('my_proj-1.2.3-dirty')
    assert is_dirty_version('my_proj-1.2.3-dirty-broken')

def convert_tuple_to_dict(tpl) -> dict:
    '''
    Converts a key-value pair tuple into a dict,
    for example:

    ((ke1, value1), (key2, value2))
    ->
    {key1: value1, key2: value2}
    '''
    dct = {}
    for key, value in tpl:
        dct[key] = value
    return dct

@click.command(context_settings=CONTEXT_SETTINGS)
@click.argument("src", type=click.File("r"))
@click.argument("dst", type=click.File("w"))
@click.argument("additional_replacements", type=replace_vars.KEY_VALLUE_PAIR, nargs=-1)
@click.option('--src-file-path', '-p',
        type=click.Path(dir_okay=False, file_okay=True),
        envvar='PROJECT_SRC_FILE_PATH',
        default=None,
        help='The path to the source file, relative to the repo root. '
            + 'This is only used in variable substitution; '
            + 'no reading from that path will be attempted. (default: SRC)')
@click.option('--repo-path', '-r',
        type=click.Path(dir_okay=True, file_okay=False),
        envvar='PROJECT_REPO_PATH',
        default='.',
        help='The path to the local git repo')
@click.option('--repo-url', '-u',
        type=click.STRING,
        envvar='PROJECT_REPO_URL',
        default=None,
        help='Public project repo URL')
@click.option('-n', '--name',
        type=click.STRING,
        envvar='PROJECT_NAME',
        default=None,
        help='Project name (prefferably without spaces)')
@click.option('--vers',
        type=click.STRING,
        envvar='PROJECT_VERSION',
        default=None,
        help='Project version (prefferably without spaces)')
@click.option('-d', '--version-date',
        type=click.STRING,
        envvar='PROJECT_VERSION_DATE',
        default=None,
        help='Date at which this version of the project was committed/released')
@click.option('--build-date',
        type=click.STRING,
        envvar='PROJECT_BUILD_DATE',
        default=None,
        help=('Date at which the currently being-made build of '
            + 'the project is made. This should basically always be left on the '
            + 'default, which is the current date.'))
@click.option('--date-format',
        type=click.STRING,
        default=DATE_FORMAT,
        help=('The format for the version and the build dates; '
            + 'see pythons date.strftime documentation'))
@click.option('--kicad-pcb',
        is_flag=True,
        help='Whether the filtered file is a *.kicab_pcb')
@click.option('--dry',
        is_flag=True,
        help='Whether to skip the actual replacing')
@click.option('--verbose',
        is_flag=True,
        help='Whether to output additional info to stderr')
def replace_single_command(
        src,
        dst,
        additional_replacements,
        src_file_path=None,
        repo_path='.',
        repo_url=None,
        name=None,
        vers=None,
        version_date=None,
        build_date=None,
        date_format=DATE_FORMAT,
        kicad_pcb=False,
        dry=False,
        verbose=False) -> None:
    '''
    Using a KiCad PCB file as input,
    replaces variables of the type `${VAR_NAME}` in text-fields with actual values,
    writing the result to an other KiCad PCB file.

    Key-value pairs to be used for the replacement are collected from 3 sources:

    * read from common environment variables like `PROJECT_REPO_PATH` and `PROJECT_REPO_URL`

    * specified through command-line switches like `--repo-url "https://github.com/user/repo/"`

    * directly specified through `ADDITIONAL_REPLACEMENTS`, for example `"PROJECT_BATCH_ID=john-1"`

    SRC - The source KiCad PCB file (this will be used as input,
    and potentially for the replacement variable `${PROJECT_SRC_FILE_PATH}`).

    DST - The destination KiCad PCB file (this will be used for the generated output).

    ADDITIONAL_REPLACEMENTS - Each one of these is a ful key-value pair,
    using '=' as the delimiter, for example `"PROJECT_BATCH_ID=john-1"`.
    '''
    add_repls_dict = convert_tuple_to_dict(additional_replacements)
    prepare_project_vars(
        add_repls_dict,
        repo_path,
        repo_url,
        name,
        vers,
        version_date,
        build_date,
        date_format,
        verbose)
    replace_single(
            src,
            dst,
            add_repls_dict,
            src_file_path,
            repo_path,
            kicad_pcb,
            dry,
            verbose)

def prepare_project_vars(
        repls: dict,
        repo_path,
        repo_url,
        name,
        vers,
        version_date,
        build_date,
        date_format,
        verbose) -> None:
    repo = Repo(repo_path)
    vcs_branch = repo.head.reference
    vcs_remote_tracking_branch = vcs_branch.tracking_branch()
    vcs_remote = vcs_remote_tracking_branch.remote_name
    if repo_url is None:
        remote_urls = repo.remotes[vcs_remote].urls
        try:
            repo_url = next(remote_urls)
        except StopIteration as err:
            raise ValueError('No remote urls defined in repo "%s"' % repo_path) from err
        if not repo_url.startswith('https://'):
            repo_url = git_remote_to_https_url(repo_url)
    if name is None:
        name = os.path.basename(os.path.abspath(repo_path))
    if vers is None:
        vers = repo.git.describe('--tags', '--dirty', '--broken', '--always')
    if version_date is None:
        version_date = date.fromtimestamp(repo.head.ref.commit.committed_date).strftime(date_format)
    if is_dirty_version(vers):
        print(f"WARNING: Dirty project version ('{vers}')! ' "
                + "(you have uncommitted changes in your project)")
    if build_date is None:
        build_date = date.today().strftime(date_format)
    repls.setdefault('PROJECT_REPO_URL', repo_url)
    repls.setdefault('PROJECT_NAME', name)
    repls.setdefault('PROJECT_VERSION', vers)
    repls.setdefault('PROJECT_VERSION_DATE', version_date)
    repls.setdefault('PROJECT_BUILD_DATE', build_date)

def replace_single(
        src,
        dst,
        replacements={},
        src_file_path=None,
        repo_path='.',
        kicad_pcb=False,
        dry=False,
        verbose=False) -> None:
    if src_file_path is None:
        src_file_path = os.path.relpath(src.name, repo_path)
    if src_file_path == '-':
        print('WARNING: "src_file_path" has the generic value "%s"'
                % src_file_path, file=sys.stderr)
    replacements.setdefault('SOURCE_FILE_PATH', src_file_path)
    if not kicad_pcb and src_file_path and src_file_path.endswith(".kicad_pcb"):
        kicad_pcb=True
        print('WARNING: Automatically set kicad_pcb=True due to appropriate file-suffix',
                file=sys.stderr)
    pre_filter=None
    post_filter=None
    if kicad_pcb:
        pre_filter=filter_kicad_quote
        post_filter=filter_kicad_unquote
        if verbose:
            print('INFO: KiCad PCB filters will be applied', file=sys.stderr)
    if kicad_pcb:
        # As we are not dealing with the (Lisp-like), raw KiCad PCB (PCBnew) syntax here,
        # but just with the actual text, we do not require pre- and post-fitlering.
        pre_filter = None
        post_filter = None
        pcb = pcbnew.LoadBoard(src.name)
        filters = replace_vars.replacements_to_filters(replacements, pre_filter, post_filter)
        verbose_loop = verbose
        for drawing in pcb.GetDrawings():
            if drawing.GetClass() == "PTEXT":
                drawing.SetText(replace_vars.filter_string(drawing.GetText(),
                    filters, dry, verbose_loop))
                # As all the text fields use the same set of replacements,
                # we at most want to print them once
                verbose_loop = False
        pcbnew.SaveBoard(dst.name, pcb)
    else:
        replace_vars.replace_vars_by_lines_in_stream(
            src, dst, replacements, dry, verbose,
            pre_filter=pre_filter, post_filter=post_filter)

@click.command(context_settings=CONTEXT_SETTINGS)
@click.argument("src_root",
        type=click.Path(exists=True, dir_okay=True, file_okay=False, readable=True))
@click.argument("glob", type=click.STRING)
@click.argument("dst_root",
        type=click.Path(exists=True, dir_okay=True, file_okay=False, writable=True))
@click.argument("additional_replacements", type=replace_vars.KEY_VALLUE_PAIR, nargs=-1)
@click.option('--src-file-path', '-p',
        type=click.Path(dir_okay=False, file_okay=True),
        envvar='PROJECT_SRC_FILE_PATH',
        default=None,
        help='The path to the source file, relative to the repo root. '
            + 'This is only used in variable substitution; '
            + 'no reading from that path will be attempted. (default: SRC)')
@click.option('--repo-path', '-r',
        type=click.Path(dir_okay=True, file_okay=False),
        envvar='PROJECT_REPO_PATH',
        default='.',
        help='The path to the local git repo')
@click.option('--repo-url', '-u',
        type=click.STRING,
        envvar='PROJECT_REPO_URL',
        default=None,
        help='Public project repo URL')
@click.option('-n', '--name',
        type=click.STRING,
        envvar='PROJECT_NAME',
        default=None,
        help='Project name (prefferably without spaces)')
@click.option('--vers',
        type=click.STRING,
        envvar='PROJECT_VERSION',
        default=None,
        help='Project version (prefferably without spaces)')
@click.option('-d', '--version-date',
        type=click.STRING,
        envvar='PROJECT_VERSION_DATE',
        default=None,
        help='Date at which this version of the project was committed/released')
@click.option('--build-date',
        type=click.STRING,
        envvar='PROJECT_BUILD_DATE',
        default=None,
        help='Date at which the currently being-made build of the project is made.'
            + ' This should basically always be left on the default, which is the current date.')
#@click.option('--recursive', '-R', type=click.STRING, default=None,
#       help='If --src-file-path points to a directory, and this is a globWhether to skip the actual replacing')
@click.option('--date-format',
        type=click.STRING,
        default=DATE_FORMAT,
        help='The format for the version and the build dates; '
                + 'see pythons date.strftime documentation')
@click.option('--kicad-pcb',
        is_flag=True,
        help='Whether the filtered file is a *.kicab_pcb')
@click.option('--dry',
        is_flag=True,
        help='Whether to skip the actual replacing')
@click.option('--verbose',
        is_flag=True,
        help='Whether to output additional info to stderr')
def replace_recursive_command(
        src_root='.',
        glob='*.kicad_pcb',
        dst_root='./build/gen-src',
        additional_replacements=(),
        src_file_path=None,
        repo_path='.',
        repo_url=None,
        name=None,
        vers=None,
        version_date=None,
        build_date=None,
        date_format=DATE_FORMAT,
        kicad_pcb=False,
        dry=False,
        verbose=False) -> None:
    '''
    Scanns for all *.kicad_pcb files in the SRC_ROOT directory,
    replaces variables of the type `${VAR_NAME}` in text-fields with actual values,
    and writes the resulting PCB to the DST_ROOT, using the same sub-path.

    Key-value pairs to be used for the replacement are collected from 3 sources:

    * read from common environment variables like `PROJECT_REPO_PATH` and `PROJECT_REPO_URL`

    * specified through command-line switches like `--repo-url "https://github.com/user/repo/"`

    * directly specified through `ADDITIONAL_REPLACEMENTS`, for example `"PROJECT_BATCH_ID=john-1"`

    Use `$${KEY}` for quoting variables you do not want expanded.

    SRC - The source KiCad PCB file (this will be used as input,
    and potentially for the replacement variable `${PROJECT_SRC_FILE_PATH}`).

    DST - The destination KiCad PCB file (this will be used for the generated output).

    ADDITIONAL_REPLACEMENTS - Each one of these is a ful key-value pair,
    using '=' as the delimiter, for example `"PROJECT_BATCH_ID=john-1"`.
    '''
    add_repls_dict = convert_tuple_to_dict(additional_replacements)
    prepare_project_vars(
        add_repls_dict,
        repo_path,
        repo_url,
        name,
        vers,
        version_date,
        build_date,
        date_format,
        verbose)
    replace_recursive(
            src_root,
            glob,
            dst_root,
            add_repls_dict,
            src_file_path,
            repo_path,
            kicad_pcb,
            dry,
            verbose)

def replace_recursive(
        src_root='.',
        glob='*.kicad_pcb',
        dst_root=None,
        add_repls_dict={},
        src_file_path=None,
        repo_path='.',
        kicad_pcb=False,
        dry=False,
        verbose=False) -> None:
    '''
    Recursively scanns a directory for KiCad PCB (pcbnew) files,
    and replaces variable keys with values in eahc one of them.
    '''
    if src_root == dst_root:
        dst_root = None
    if verbose:
        print('INFO: Scanning directory "%s" for "%s" ...' % (src_root, glob), file=sys.stderr)
    if dst_root:
        dst_root_abs = os.path.abspath(dst_root)
    for path in Path(src_root).rglob(glob):
        if dst_root and os.path.commonpath([os.path.abspath(path), dst_root_abs]) == dst_root_abs:
            # Exclude files in the dst_root
            continue
        src_path = str(path)
        if dst_root:
            dst_path = os.path.join(dst_root, os.path.relpath(src_path, src_root))
            os.makedirs(os.path.dirname(os.path.abspath(dst_path)), exist_ok=True)
        else:
            dst_path = src_path + ".TMP"
        if verbose:
            if dst_root:
                print('INFO: Processing file from "%s" -> "%s" ...'
                        % (src_path, dst_path), file=sys.stderr)
            else:
                print('INFO: Processing file "%s" ...' % src_path, file=sys.stderr)
        src = click.open_file(src_path, "r")
        dst = click.open_file(dst_path, "w")
        replace_single(src, dst, add_repls_dict, src_file_path, repo_path,
                kicad_pcb, dry, verbose)
        if not dst_root:
            os.rename(dst.name, src.name)

if __name__ == '__main__':
    replace_single_command()
    #replace_recursive_command()
    #test_is_dirty_version()
