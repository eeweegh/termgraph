"""Utility functions for the project.
"""


import sys
import tomllib
from pathlib import Path
from typing import cast

from math import floor, log

""" types
"""
Number = int | float
ArgsType = dict[str, str | Number | bool | list[str] | None]

def get_version() -> str:
    """Get the version of the project from pyproject.toml file.
    """
    try:
        with Path("pyproject.toml").open("rb") as f:
            return cast(str, tomllib.load(f)["project"]["version"])
    except (FileNotFoundError, KeyError):
        return "UNKNOWN"


def cvt_to_readable(num: Number) -> tuple[Number, str]:
    """
    Convert a number into a human‑readable format.

    Examples:
        125000  → 125.0K
        12550   → 12.55K
        19561100 → 19.561M
    """


    UNITS = ["", "K", "M", "B", "T"]

    # Zero case
    if num == 0:
        return (0.0, UNITS[0])

    # Work with absolute value for formatting
    abs_num = abs(num)

    # Determine magnitude (thousands, millions…)
    index = int(floor(log(abs_num, 1000)))
    index = min(index, len(UNITS) - 1)  # prevent overflow

    # Compute scaled value
    scaled: float = cast(float, abs_num / (1000 ** index))

    # Round to 3 decimal places
    result = round(scaled, 3)

    # Apply sign if needed
    if num < 0:
        result = -result

    return (result, UNITS[index])

def dayname(num: int) -> str:
    """Convert a day number to a string (0 is Monday)

    """
    DAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
    return DAYS[num % 7]

DELIMITER: str = ","

# this one is nice '🟥' (from Unicode Geometric Shapes Block), but double charwidth, so needs correction in calculation
TICK: str = "⣿"
SMALLTICK: str = "⡇"

# 8 bit ANSI escape color codes, selenized
# first one is default
COLORS = {
    "green": 2,
    "red": 1,
    "yellow": 3,
    "blue": 4,
    "magenta": 5,
    "cyan": 6,
    "grey": 7,
    "black": 0,
    "brightblack": 8,
    "brightred": 9,
    "brightgreen": 10,
    "brightyellow": 11,
    "brightblue": 12,
    "brightmagenta": 13,
    "brightcyan": 14,
    "brightwhite": 15,
    "24": 24,
    "25": 25,
    "26": 26,
    "27": 27,
    "28": 28,
    "29": 29,
    "30": 30,
    "31": 31,
    "32": 32,
    "33": 33,
    "34": 34,
    "35": 35,
    "36": 36,
    "37": 37,
    "38": 38,
    "39": 39,
}

# print a color table
def print_color_table():
    """Print a color table for 8 bit ANSI escape codes.
    """
    for x in range(32):
        for y in range(8):
            color = x * 8 + y
            _=sys.stdout.write("\033[38:5:{}m{:>3d} ▇▇▇▇▇\033[0m  ".format(color, color))
        print()

# Prints a row of the horizontal graph.
def print_row(
        value: Number,
        num_blocks: int,
        val_min: Number,
        color: int | None = None,
        label: str = "",
        tail: str = "",
        doprint: bool = False,
        tick: str = TICK,
        smalltick: str = SMALLTICK
        ):
    """A method to print a row for a horizontal graphs.
    i.e:
    1 ▇▇ 2
    2 ▇▇▇ 3
    3 ▇▇▇▇ 4
    """
    _ = sys.stdout.write("\033[0m")  # no color

    if doprint:
        print(label, tail, " ", end="")

    if color:
        _ = sys.stdout.write(f"\033[38:5:{color}m")  # Start to write colorized.

    if (num_blocks < 1 and (value > val_min or value > 0)) or (
        doprint and value == 0.0
    ):
        # Print something if it's not the smallest
        # and the normal value is less than one.
        _ = sys.stdout.write(smalltick)
    else:
        for _ in range(num_blocks):
            _ = sys.stdout.write(tick)

    if color:
        _ = sys.stdout.write("\033[0m")  # Back to original.

    if doprint:
        print()

