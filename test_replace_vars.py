# SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
#
# SPDX-License-Identifier: GPL-3.0-or-later

import unittest

from click.testing import CliRunner

from replace_vars import cli

class TestReplaceVars(unittest.TestCase):
    def test_replace_vars(self):
        template = 'Try $this, ${this, $this}, $${this}, ${$this}, or ${this}$.\nNo, ${this} ${this}!\n'
        expected = 'Try $this, ${this, $this}, ${this}, ${$this}, or that$.\nNo, that that!\n'
        runner = CliRunner()
        result = runner.invoke(cli, ['-', '-', 'this:that'], input=template)
        self.assertEqual(result.exit_code, 0)
        self.assertEqual(result.output, expected)

if __name__ == '__main__':
    TestReplaceVars().test_replace_vars()
