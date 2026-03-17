# FIXME: globals for vertical, not ideal
value_list, zipped_list, vertical_list, maxi = [], [], [], 0



def calendar_heatmap(data: list, labels: list, args: dict) -> None:
    """Print a calendar heatmap."""
    if args["color"]:
        colornum = COLORS.get(args["color"][0])
    else:
        colornum = COLORS.get("blue")

    dt_dict = {}
    for i in range(len(labels)):
        dt_dict[labels[i]] = data[i][0]

    # get max value
    max_val = float(max(data)[0])

    tick_1 = "░"
    tick_2 = "▒"
    tick_3 = "▓"
    tick_4 = "█"

    if args["custom_tick"]:
        tick_1 = tick_2 = tick_3 = tick_4 = args["custom_tick"]

    # check if start day set, otherwise use one year ago
    if args["start_dt"]:
        start_dt = datetime.strptime(args["start_dt"], "%Y-%m-%d")
    else:
        start = datetime.now()
        start_dt = datetime(year=start.year - 1, month=start.month, day=start.day)

    # modify start date to be a Monday, subtract weekday() from day
    start_dt = start_dt - timedelta(start_dt.weekday())

    # TODO: legend doesn't line up properly for all start dates/data
    # top legend for months
    sys.stdout.write("     ")
    for month in range(13):
        month_dt = datetime(
            year=start_dt.year, month=start_dt.month, day=1
        ) + timedelta(days=month * 31)
        sys.stdout.write(month_dt.strftime("%b") + " ")
        if args["custom_tick"]:  # assume custom tick is emoji which is one wider
            sys.stdout.write(" ")

    sys.stdout.write("\n")

    for day in range(7):
        sys.stdout.write(dayname([day]) + " ")
        for week in range(53):
            day_ = start_dt + timedelta(days=day + week * 7)
            day_str = day_.strftime("%Y-%m-%d")

            if day_str in dt_dict:
                if dt_dict[day_str] > max_val * 0.75:
                    tick = tick_4
                elif dt_dict[day_str] > max_val * 0.50:
                    tick = tick_3
                elif dt_dict[day_str] > max_val * 0.25:
                    tick = tick_2
                # show nothing if value is zero
                elif dt_dict[day_str] == 0.0:
                    tick = " "
                # show values for less than 0.25
                else:
                    tick = tick_1
            else:
                tick = " "

            if colornum:
                sys.stdout.write("\033[38:5:{colornum}m".format(colornum=colornum))

            sys.stdout.write(tick)
            if colornum:
                sys.stdout.write("\033[0m")

        sys.stdout.write("\n")

def hist_rows(data: list, args: dict, colors: list):
    """Prepare the Histogram graph.
    Each row is printed through the print_row function."""

    val_min = find_min(data)
    val_max = find_max(data)

    # Calculate borders
    class_min = floor(val_min)
    class_max = ceil(val_max)
    class_range = class_max - class_min
    class_width = class_range / args["bins"]

    border = class_min
    borders = []
    max_len = len(str(border))

    for b in range(args["bins"] + 1):
        borders.append(border)
        len_border = len(str(border))
        if len_border > max_len:
            max_len = len_border
        border += class_width
        border = round(border, 1)

    # Count num of data via border
    count_list = []

    for start, end in zip(borders[:-1], borders[1:]):
        count = 0
        #        for d in [d]
        for v in [row[0] for row in data]:
            if start <= v < end:
                count += 1

        count_list.append([count])

    normal_counts = normalize(count_list, args["width"])

    for i, border in enumerate(zip(borders[:-1], borders[1:])):
        if colors:
            color = colors[0]
        else:
            color = None

        if not args.get("no_labels"):
            print("{:{x}} – {:{x}} ".format(border[0], border[1], x=max_len), end="")

        num_blocks = normal_counts[i]

        yield (count_list[i][0], int(num_blocks[0]), 0, color)

        if args.get("no_values"):
            tail = ""
        else:
            tail = " {}{}".format(
                args["format"].format(count_list[i][0]), args["suffix"]
            )
        print(tail)


def print_vertical(vertical_rows: list, labels: list, color: bool, args: dict) -> None:
    """Print the whole vertical graph."""
    if color:
        _ = sys.stdout.write( "\033[38:5:{color}m".format(color=color))  # Start to write colorized.

    for row in vertical_rows:
        print(*row)

    sys.stdout.write("\033[0m")  # End of printing colored

    if not args["no_values"]:
        print("-" * len(row) + "Values" + "-" * len(row))
        for value in zip_longest(*value_list, fillvalue=" "):
            print("  ".join(value))

    if not args["no_labels"]:
        print("-" * len(row) + "Labels" + "-" * len(row))
        # Print Labels
        for label in zip_longest(*labels, fillvalue=""):
            print("  ".join(label))

def vertically(value, num_blocks: int, val_min: int, color: bool, args: dict) -> list:
    """Prepare the vertical graph.
    The whole graph is printed through the print_vertical function."""
    global maxi, value_list

    value_list.append(str(value))

    # In case the number of blocks at the end of the normalization is less
    # than the default number, use the maxi variable to escape.
    if maxi < num_blocks:
        maxi = num_blocks

    if num_blocks > 0:
        vertical_list.append((tick * num_blocks))
    else:
        vertical_list.append(smalltick)

    # Zip_longest method in order to turn them vertically.
    for row in zip_longest(*vertical_list, fillvalue=" "):
        zipped_list.append(row)

    counter, result_list = 0, []

    # Combined with the maxi variable, escapes the appending method at
    # the correct point or the default one (width).
    for i in reversed(zipped_list):
        result_list.append(i)
        counter += 1

        if maxi == args["width"]:
            if counter == (args["width"]):
                break
        else:
            if counter == maxi:
                break

    # Return a list of rows which will be used to print the result vertically.
    return result_list

