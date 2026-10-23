#!/usr/bin/env zsh
# Shows off termgraph and the tg driver on plain CSV. Run with `just demo`.

set -o errexit; set -o nounset; set -o pipefail

root=${0:A:h:h}
cargo build --release --locked --quiet --manifest-path ${root}/Cargo.toml
export PATH=${root}/target/release:${root}/bin:${PATH}

show() {
   print -P "\n%B%F{blue}# ${1//\%/%%}%f%b"
   print -P "%F{244}\$ ${2//\%/%%}%f"
   eval "${2}"
}

births=$'year,Boys,Girls\n2007,183,202\n2008,231,199\n2009,250,240\n2010,190,260\n2011,310,280'
single=$'month,sales\nJan,120\nFeb,340\nMar,275\nApr,610\nMay,480'
cities=$'city,people\nGhent,265000\nAntwerp,530000\nBrussels,1220000'
bare=$'a,3\nb,7\nc,5'
mixed=$'region,q1,q2,q3\nnorth,-20,40,65\nsouth,35,-10,50\neast,80,90,-30\nwest,12,12,12'

show "plain csv, one series: no legend, header is just a header" \
   'print -r -- $single | termgraph plot --color green --width 40'

show "several series, grouped" \
   'print -r -- $births | termgraph plot --width 40'

show "tg: stacked, sized to the terminal" \
   'print -r -- $births | tg'

show "tg: TGNOSTACK=1 for grouped bars" \
   'print -r -- $births | TGNOSTACK=1 tg'

show "tg: options pass through (colors, title, format)" \
   'print -r -- $births | tg --color blue red --title Births --format %.0f'

show "negative values" \
   'print -r -- $mixed | tg'

show "big numbers get K/M/B scaling" \
   'print -r -- $cities | termgraph plot --width 40 --color yellow'

show "legend on request for a single series" \
   'print -r -- $single | termgraph plot --legend --width 40'

show "custom tick" \
   'print -r -- $single | termgraph plot --custom-tick 🔥 --width 30'

show "headerless input still works" \
   'print -r -- $bare | termgraph plot --width 30 --color red'
