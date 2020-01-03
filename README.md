# quaestor: manage things for consul

Quastor is the lowest-ranking position on the cursus honorum, and consul is the highest postion. The root word means "to inquire". Quaestor is a command-line tool for working with consul key value store items in handy ways, particularly if you don't plan on having the consul command-line client around.

If the environment variable `CONSUL_HTTP_ADDR` is set, quaestor will use it. If not present, it falls back to `http://localhost:8500/`. Note that this must be a full URI with schema, port, and trailing slash.

Commands:

* `quaestor set <key> <value>` - set the given key to the passed-in value.
* `quaestor get <key>` - get the value of the given key, in plain text.
* `quaestor rm <key>` - remove the key from consul
* `quaestor dir <key>` - treat the key as a prefix and get all keys underneath it, in plain text.
* `quaestor export` - export all key/value pairs as json to stdout.
* `quaestor import <file>` - import all key/value pairs in the given json input file; reads from stdin if you pass `-` as the file name.

## TODO

* Make the tool more forgiving about trailing slashes on CONSUL_HTTP_ADDR.
* Better usage examples and help text.

Stretch goals:

* `quaestor mirror <dburl>` - mirror the contents of an existing consul

## LICENSE

MIT

