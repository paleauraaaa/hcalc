# Commands:
* `a[dd] <name>: <coords>` - add new firing position
* `n[ew] <name>: <coords>` - add new fire mission
* `e[dit] <name>: <coords>` - edit an existing fire mission
* `d[elete] <name>` - delete a fire mission
* `l[ist]` - list existing fire missions, with bearing, distance, mils, and charge type from all existing firing positions
* `r[eset]` - clear all fire missions and firing positions
* `q[uit]` - exit the program

`name` may contain spaces and may be quoted, but may not contain a colon, as that serves as the delimiter.
`coords` may be comma-separated, space-separated, or both, and may optionally be parenthesized. That is, `500 1000`, `500, 1000`, `(500 1000)`, and `(500, 1000)` would all be valid coords.
