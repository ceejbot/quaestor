# quaestor: manage things for consul

Implemented:

* `quaestor set key value` - set the given key to the passed-in value
* `quaestor get key` - get the value of the given key
* `quaestor rm key` - remove the key from consul
* `quaestor dir key` - treat the key as a prefix
* `quaestor export` - export all key/value pairs as json

Todo:

* Read consul location from env var instead of hardcoding to http://localhost:8500/.

Stretch goals:

* `quaestor import file` - import all key/value pairs in the given json input file
* `quaestor mirror dburl` - mirror the contents of an existing consul

## LICENSE

MIT

