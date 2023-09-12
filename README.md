# Wasmtime Component Model Val Serialization

`serde` (de)serialization for
[`wasmtime::component::Val`s](https://docs.rs/wasmtime/latest/wasmtime/component/enum.Val.html)

## JSON mapping

> While this crate implements generic `serde` interfaces and thus could plausibly
work with other formats, it has been built with JSON in mind.

### Basic Types

| Component Model Type | JSON Type
| --- | ---
| Booleans (`bool`) | `bool`
| Integers (`{s,u}{8,16,32,64}`) | `number` or `string` (base 10)
| Floats (`float{32,64}`) | `number` or one of `"NaN"`, `"Infinity"`, `"-Infinity"`
| Characters (`char`) | `string` (single [Unicode Scalar Value](https://unicode.org/glossary/#unicode_scalar_value))
| Strings (`string`) | `string`
| Options (`option<T>`) | `<T>` or `null` (*)
| Lists (`list<T>`) | `[<T>, <T>, ...]`
| Tuples (`tuple<T0, T1, ...>`) | `[<T0>, <T1>, ...]`

> (*) Nested `options`, e.g. `option<option<T>>` are handled specially, with
> outer `some` values serialized as `{"value": <T or null>}`.

### Records

A `record` is serialized as a JSON object, with `kebab-case` keys.
`option` fields may be omitted if their value is `none`/`null`.

### Variants

A `variant` is serialized as a JSON object with a single entry,
where the key of the entry is the variant case's `kebab-case` name. For cases
without payloads, the value is `null`.

### Enums

An `enum` is serialized as a JSON string with the enum case's `kebab-case` name.

### Flags

A `flags` is serialized as a JSON array with the flags' `kebab-case` names.

### Results

A `result<T, E>` is serialized as either `{"result": <T>}` or `{"error": <E>}`.
If the `result` does not have an `ok` or `err` payload, the corresponding value
is `null`.

> TODO: examples
