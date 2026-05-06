# Commands:
* `a[dd] <name> <coords>` - add new firing position
* `n[ew] <name> <coords>` - add new fire mission
* `e[dit] <name> <coords>` - edit an existing fire mission
* `d[elete] <name>` - delete a fire mission
* `l[ist]` - list existing fire missions, with bearing, distance, mils, and charge type from all existing firing positions
* `r[eset]` - clear all fire missions and firing positions
* `q[uit]` - exit the program

`name` must be either a single word (no spaces), or a quoted string (may include spaces). For example, `single_word` or `"Multiple Words"`.
`coords` may be comma-separated, space-separated, or both, and may optionally be parenthesized. That is, `500 1000`, `500,1000`, `500, 1000`, `(500 1000)`, `(500,1000)`, and `(500, 1000)` would all be valid coords.

# Build
Pre-compiled binaries for Windows and Linux are provided, but `hcalc` should build for any target with a working `std` environment with a simple `cargo build`. I'd be happy to provide pre-compiled macos executables too, but alas, I do not own an Apple device.
