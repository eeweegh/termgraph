

import sys
import argparse

from .utils import Number


ArgsType = dict[str, str | Number | bool | list[str] | None]

def init_args() -> ArgsType:
    """Parse and return the arguments."""
    parser = argparse.ArgumentParser(description="draw basic graphs on terminal")

    _ = parser.add_argument("filename", nargs="?", default="-", help="data file name (comma or space separated). Defaults to stdin.")
    _ = parser.add_argument("--title", help="Title of graph")
    _ = parser.add_argument("--width", type=int, default=50, help="width of graph in characters default:50")
    _ = parser.add_argument("--format", default="{:<5.2f}", help="format specifier to use.")
    _ = parser.add_argument("--suffix", default="", help="string to add as a suffix to all data points.")
    _ = parser.add_argument("--no-labels", action="store_true", help="Do not print the label column")
    _ = parser.add_argument("--no-values", action="store_true", help="Do not print the values at end")
    _ = parser.add_argument("--space-between", action="store_true", help="Print a new line after every field",)
    _ = parser.add_argument("--color", nargs="*", help="Graph bar color(s)")
    _ = parser.add_argument("--stacked", action="store_true", help="Stacked bar graph")
    _ = parser.add_argument("--differentscale", action="store_true", help="Categories have different scales.",)
    _ = parser.add_argument("--custom-tick", default="", help="Custom tick mark, emoji approved")
    _ = parser.add_argument("--delim", default="", help="Custom delimiter, default , or space")
    _ = parser.add_argument("--verbose", action="store_true", help="Verbose output, helpful for debugging")
    _ = parser.add_argument("--label-before", action="store_true", default=False, help="Display the values before the bars",)
    _ = parser.add_argument("--version", action="store_true", help="Display version and exit")
    _ = parser.add_argument("--colors", action="store_true", help="show color table used")

    if len(sys.argv) == 1:
        if sys.stdin.isatty():
            parser.print_usage()
            sys.exit(2)

    args: ArgsType = vars(parser.parse_args())


    return args


