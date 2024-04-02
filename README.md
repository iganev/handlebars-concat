[![Crates.io](https://img.shields.io/crates/v/handlebars-concat?color=4d76ae)](https://crates.io/crates/handlebars-concat)
[![API](https://docs.rs/handlebars-concat/badge.svg)](https://docs.rs/handlebars-concat)
[![dependency status](https://deps.rs/repo/github/iganev/handlebars-concat/status.svg)](https://deps.rs/repo/github/iganev/handlebars-concat)
[![build and test](https://github.com/iganev/handlebars-concat/actions/workflows/rust.yml/badge.svg)](https://github.com/iganev/handlebars-concat/actions/workflows/rust.yml)
[![codecov](https://codecov.io/github/iganev/handlebars-concat/graph/badge.svg?token=B5P2TAV5BB)](https://codecov.io/github/iganev/handlebars-concat)

# handlebars-concat
String, Array and Object concatenator helper for [handlebars-rust](https://github.com/sunng87/handlebars-rust)

## Quick Start

Developed and tested with handlebars-rust v4.4.0.  
Version `0.1.*` are compatible with handlebars `4`.  
Version `0.2.*` are compatible with handlebars `5`. (Thanks to [campeis](https://github.com/campeis))

### Registration

```rust
    use handlebars::Handlebars;
    use handlebars_concat::HandlebarsConcat;
    
    let mut h = Handlebars::new();
    h.register_helper("concat", Box::new(HandlebarsConcat));
```

### Behavior

The helper is looking for multiple arguments of type string, array or object. Arguments are being added to an output buffer and returned altogether as string.   

The helper has few parameters modifying the behavior slightly. For example `distinct=true` eliminates duplicate values from the output buffer, while `quotes=true` in combination with `single_quote=true` wraps the values in quotation marks. See [Parameters](#parameters) for more. 

#### String
~~String arguments are added directly to the output buffer.~~  
As of `0.1.3` strings could be handled in one of two ways:
1. By default strings are added to the output buffer without modification (other than the quotation mark modifiers).
2. If you add a block template and use the `render_all` parameter, strings will be passed as `{{this}}` to the block template.  

The block template rendering is disabled by default for backward compatibility.

#### Array
~~Array arguments are iterated and added as individual strings to the output buffer.~~  
As of `0.1.3` arrays could be handled in one of two ways:
1. By default array values are added as individual strings to the output buffer without modification (other than the quotation mark modifiers).
2. If you add a block template and use the `render_all` parameter, array values are passed as `{{this}}` to the block template.  

The block template rendering is disabled by default for backward compatibility.

#### Object
Object arguments could be handled two different ways:
1. By default only the object keys are being used and the values are ignored.
2. If you add a block template the helper will use it to render the object value and  
concatenate it as string to the output buffer.

Object rendering results are subject to `distinct`, `quotes` and `single_quote` modifier parameters, just like strings and arrays.  

### Parameters

The helper accepts several hash arguments to modify the concatenation behavior:
- `separator`: Set specific string to join elements with. Default is ","
- `distinct`: Eliminate duplicates upon adding to output buffer
- `quotes`: Wrap each value in double quotation marks
- `single_quote`: Modifier of `quotes` to switch to single quotation mark instead
- `render_all`: Render all values using the block template, not just object values

### Examples

Example with string literals:

```handlebars
{{concat "One" "Two" separator=", "}}
```

Result: `One, Two`

---

```handlebars
{{concat "One" "Two" separator=", " quotes=true}}
```

Result: `"One", "Two"`

---

Where `s` is `"One"`, `arr` is `["One", "Two"]` and `obj` is `{"Three":3}`

```handlebars
{{concat s arr obj separator=", " distinct=true}}
```

Result: `One, Two, Three`

---

Where `s` is `"One"`, `arr` is `["One", "Two"]` and `obj` is `{"key0":{"label":"Two"},"key1":{"label":"Three"},"key2":{"label":"Four"}}`:

```handlebars
{{#concat s arr obj separator=", " distinct=true}}{{label}}{{/concat}}
```

Result: `One, Two, Three, Four`

---

Where `s` is `"One"`, `arr` is `["One", "Two"]` and `obj` is `{"key0":{"label":"Two"},"key1":{"label":"Three"},"key2":{"label":"Four"}}`

```handlebars
{{#concat s arr obj separator=", " distinct=true render_all=true}}<{{#if label}}{{label}}{{else}}{{this}}{{/if}}/>{{/concat}}
```

Result: `<One/>, <Two/>, <Three/>, <Four/>`

---

## License

This library (handlebars-concat) is open sourced under the BSD 2 License.  
