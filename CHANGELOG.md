# Changelog

## v0.5.1 (2025-12-15)

* Use full-path for serialize trait method to avoid collisions with other
  serialize trait methods
* Fix unused assignment warnings

## v0.5.0 (2023-10-03)

* Use indexmap v2.x.y

## v0.4.0 (2023-03-10)

* Avoid allocating strings when serializing static field/variant names
* Redesign trait bounds for map keys

## v0.3.2 (2022-07-13)

* Add support for serialize_with, deserialize_with and update_with attributes

## v0.3.1 (2022-04-10)

* Add missing documentation link

## v0.3.0 (2022-04-10)

* Allow inlining where it makes sense in order to let the compiler to make more
  optimizations and generate less instructions
* Construct errors without any allocation where possible in order to generate
  smaller serialize/deserialize/update methods
* Use LinkedList for field error lists because it has smaller footprint (the
  performance impact is irrelevant in this case as the number of errors is
  usually quite small)
* Use a Map wrapper for the underlying hash map implementation to prevent
  inlining of some methods
* Optimize the derive macros to avoid creating collections where it is not
  necessary
* Use unwrap_unchecked() instead of unwrap() in some cases in order to avoid
  generating panic handlers in the resulting serialize/deserialize/update
  methods

## v0.2.0 (2021-09-06)

* Fix serialization/deserialization/update for externally tagged enums (see #1
  for more info)

## v0.1.1 (2021-02-02)

* Do not emit warnings for unused variables in derive Deserialize, Serialize
  and Update

## v0.1.0 (2021-01-21)

* Initial release
