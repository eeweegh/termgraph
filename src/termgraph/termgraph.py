#!/usr/bin/env python3
"""This module allows drawing basic graphs in the terminal."""

# original source: https://github.com/mkaz/termgraph

import argparse
import sys
import os
import re

from typing import cast

from .utils import get_version, DELIMITER, TICK, SMALLTICK, COLORS, print_row, cvt_to_readable, print_color_table
from .utils import Number, ArgsType



""" globals
"""
delimiter: str = DELIMITER
tick: str = TICK
smalltick: str = SMALLTICK

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

    if args["custom_tick"] != "":
        global tick, smalltick
        tick = str(args["custom_tick"])
        smalltick = ""

    if args["delim"] != "":
        global delimiter
        delimiter = str(args["delim"])

    return args




def find_min(data: list[list[Number]]) -> Number:
    """Return the minimum value in sublist of list."""
    return min([min(sublist) for sublist in data])

def find_max(data: list[list[Number]]) -> Number:
    """Return the maximum value in sublist of list."""
    return max([sum(sublist) for sublist in data])


def normalize(data: list[list[Number]], width: int) -> list[list[Number]]:
    """normalize the data and return it.
       offset by the minimum if there's a negative.
    """
    data_offset: list[list[Number]] = []
    min_datum = find_min(data)
    if min_datum < 0:
        min_datum = abs(min_datum)
        for datum in data:
            data_offset.append([d + min_datum for d in datum])
    else:
        data_offset = data

    max_datum = find_max(data_offset)

    if max_datum == 0:
        return data_offset

    else:
        # max_datum / width is the value for a single tick.
        # norm_factor is the inverse of this value
        # If you divide a number by the value of single tick, you will find how
        # many ticks it does contain basically.
        norm_factor = width / float(max_datum)
        normal_data: list[list[Number]] = []
        for datum in data_offset:
            normal_data.append([v * norm_factor for v in datum])

        return normal_data


def find_max_label_length(labels: list[str]) -> int:
    """Return the maximum length for the labels."""
    return max([len(label) for label in labels])



def stacked_graph(
        labels: list[str],
        data: list[list[Number]],
        normal_data: list[list[Number]],
        args: ArgsType,
        colors: list[int],
    ):
    """Prepare the horizontal stacked graph.
    Each row is printed through the print_row function.
    """
    val_min = find_min(data)

    for i in range(len(labels)):
        if args["no_labels"]:
            # Hide the labels.
            label = ""
        else:
            label = "{:<{x}} ".format(labels[i], x=find_max_label_length(labels))

        if args.get("space_between") and i != 0:
            print()

        print(label, end="")

        values = data[i]
        num_blocks = normal_data[i]

        for j in range(len(values)):
            print_row(values[j], int(num_blocks[j]), val_min,
                      colors[j % len(colors)], tick=tick, smalltick=smalltick)

        tail = " {}{}".format(str(args["format"]).format(sum(values)), args["suffix"])
        print(tail)



def check_data(labels: list[str], data: list[list[Number]], args: ArgsType) -> list[int]:
    """check that all data were inserted correctly. Return the colors."""
    if len(data) == 0:
        # no data to plot.
        sys.exit(0)

    len_categories = len(data[0])

    # Check that there are data for all labels.
    if len(labels) != len(data):
        print(">> Error: Label and data array sizes don't match", file=sys.stderr)
        sys.exit(1)

    # Check that there are data for all categories per label.
    for dat in data:
        if len(dat) != len_categories:
            print(">> Error: There are missing values", file=sys.stderr)
            sys.exit(1)

    colors: list[int] = []

    # If user inserts colors, they should be as many as the categories.
    argcolors: list[str] = cast(list[str], args['color'])
    if argcolors:
        if len(argcolors) != len_categories:
            print(f'>> Error: color {len(argcolors)} and category array sizes {len_categories} dont match', file=sys.stderr)
            sys.exit(1)

        for color in argcolors:
            if color not in COLORS:
                print(
                    f'>> Error: invalid color {color}. choose from red, blue, green, magenta, yellow, black, cyan',
                    file=sys.stderr)
                sys.exit(1)

        for color in argcolors:
            colors.append(cast(int, COLORS.get(color)))

    # If user hasn't inserted colors, pick the first n colors
    # from the dict (n = number of categories).
    if not colors:
        colors = [v for v in list(COLORS.values())[:len_categories]]

    return colors


def print_categories(categories: list[str], colors: list[int], args: ArgsType) -> None:
    """Print a tick and the category's name for each category above
    the graph."""
    sofar: int = 0
    argwidth: int = cast(int, args["width"])
    for i in range(len(categories)):
        if colors:
            _ = sys.stdout.write( f"\033[38:5:{colors[i % len(colors)]}m")  # Start to write colorized.

        _ = sys.stdout.write(tick + " " + categories[i] + " ")

        if colors:
            _ = sys.stdout.write("\033[0m")  # Back to original.

        sofar += len(categories[i]) + 2
        if sofar > argwidth:
            _ = sys.stdout.write("\n")
            sofar = 0

    print("\n")


def read_data(args: ArgsType) -> tuple[list[str], list[str], list[list[Number]], list[int]]:
    """Read data from a file or stdin and returns it.

    Filename includes (optional: categories), labels and data.
    - categories and labels get collected in lists.
    - data are inserted in a list of lists, one per category

    i.e.
    labels = ['2001', '2002', '2003', ...]
    categories = ['boys', 'girls']
    data = [ [20.4, 40.5], [30.7, 100.0], ...]"""

    labels: list[str] = []
    categories: list[str] = []
    data: list[list[Number]] = []

    filename: str = cast(str, args["filename"])
    stdin = filename == "-"

    if args["verbose"]:
        print(">> Reading data from {src}".format(src=("stdin" if stdin else filename)), file=sys.stderr)

    print("")
    if args["title"]:
        print("# " + cast(str, args["title"]) + "\n")

    categories, labels, data, colors = ([] for _ in range(4))

    f = None
    try:
        f = sys.stdin if stdin else open(filename, "r")
        for line in f:
            line = line.strip()
            if line:
                # simple comment syntax
                if not line.startswith("#"):
                    if line.find(delimiter) > 0:
                        # https://stackoverflow.com/questions/2785755/how-to-split-but-ignore-separators-in-quoted-strings-in-python
                        # remove delimiters, but only when not within quotes, and part of the column header
                        cols = re.split(
                            delimiter + """(?=(?:[^'"]|'[^']*'|"[^"]*")*$)""", line
                        )
                    else:
                        cols = line.split()

                    # Line contains categories.
                    if line.startswith("@"):
                        categories = [e.strip(' "') for e in cols[1:]]

                    # Line contains label and values.
                    else:
                        # remove any quotes and spaces from labelname
                        labels.append(cols[0].strip(' "'))
                        data_points: list[Number] = []
                        for i in range(1, len(cols)):
                            s = cols[i].strip()
                            if not s:
                                s = "0"
                            data_points.append(float(s))

                        data.append(data_points)

    except FileNotFoundError:
        print( f">> Error: The specified file [{filename}] does not exist.", file=sys.stderr)
        sys.exit(1)
    except IOError:
        print("An IOError has occurred!", file=sys.stderr)
        sys.exit(1)
    finally:
        if f is not None:
            _ = f.close()

    # Check that all data are valid. (i.e. There are no missing values.)
    colors = check_data(labels, data, args)
    if categories:
        # Print categories' names above the graph.
        print_categories(categories, colors, args)

    return categories, labels, data, colors

def horiz_rows(
    labels: list[str],
    data: list[list[Number]],
    normal_dat: list[list[Number]],
    args: ArgsType,
    colors: list[int]
):
    """Prepare the horizontal graph.
    Each row is printed through the print_row function."""
    val_min = find_min(data)

    for i in range(len(labels)):
        if args["no_labels"]:
            # Hide the labels.
            label = ""
        else:
            if args.get("label_before"):
                fmt = "{:<{x}}"
            else:
                fmt = "{:<{x}} "
            label = fmt.format(labels[i], x=find_max_label_length(labels))

        values = data[i]
        num_blocks: list[int] = cast(list[int], normal_dat[i])

        if args.get("space_between") and i != 0:
            print()

        for j in range(len(values)):
            # In Multiple series graph 1st category has label at the beginning,
            # whereas the rest categories have only spaces.
            if j > 0:
                len_label = len(label)
                label = " " * len_label
            if args.get("label_before"):
                fmt = "{}{}{}"
            else:
                fmt = " {}{}{}"

            if args["no_values"]:
                tail = cast(str, args["suffix"])
            else:
                val, deg = cvt_to_readable(values[j])
                tail = fmt.format(cast(str, args["format"]).format(val), deg, args["suffix"])

            if colors:
                color = colors[j]
            else:
                color = None

            if not args.get("label_before"):
                print(label, end="")

            yield (
                values[j],
                int(num_blocks[j]),
                val_min,
                color,
                label,
                tail,
                cast(bool, args.get("label_before")),
                tick,
                smalltick)

            if not args.get("label_before"):
                print(tail)

def chart(colors: list[int], data: list[list[Number]], args: ArgsType, labels: list[str]) -> None:
    """Handle the normalization of data and the printing of the graph."""
    len_categories = len(data[0])
    if len_categories > 0:
        # Stacked graph
        if args["stacked"]:
            normal_dat = normalize(data, cast(int, args["width"]))
            stacked_graph(labels, data, normal_dat, args, colors)
            return

        if not colors:
            colors = [0] * len_categories

        # Multiple series graph with different scales
        # Normalization per category
        if args["differentscale"]:
            for i in range(len_categories):
                cat_data: list[list[Number]] = []
                for dat in data:
                    cat_data.append([dat[i]])

                # Normalize data, handle negatives.
                normal_cat_data = normalize(cat_data, cast(int, args["width"]))

                # Generate data for a row.
                for row in horiz_rows(labels, cat_data, normal_cat_data, args, [colors[i % len(colors)]]):
                    # Print the row
                    print_row(*row)

                print()
            return


    # One category/Multiple series graph with same scale
    # All-together normalization
    if not args["stacked"]:
        normal_dat = normalize(data, cast(int, args["width"]))
        _ = sys.stdout.write("\033[0m")  # no color
        for row in horiz_rows(labels, data, normal_dat, args, colors):
            print_row(*row)

        print()


def main():
    """Main function."""
    args = init_args()

    if args["version"]:
        print("termgraph v{}".format(get_version()), os.name)
        sys.exit()

    if args["colors"]:
        print_color_table()
        sys.exit()

    _, labels, data, colors = read_data(args)
    try:
        chart(colors, data, args, labels)
    except BrokenPipeError:
        pass


if __name__ == "__main__":
    main()
