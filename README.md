[![Crates.io](https://img.shields.io/crates/v/handlebars-concat?color=4d76ae)](https://crates.io/crates/handlebars-concat)
[![API](https://docs.rs/handlebars-concat/badge.svg)](https://docs.rs/handlebars-concat)
[![Build and Test](https://github.com/iganev/handlebars-concat/actions/workflows/rust.yml/badge.svg)](https://github.com/iganev/handlebars-concat/actions/workflows/rust.yml)
[![dependency status](https://deps.rs/repo/github/iganev/handlebars-concat/status.svg)](https://deps.rs/repo/github/iganev/handlebars-concat)

# handlebars-concat
String, Array and Object concatenator helper for [handlebars-rust](https://github.com/sunng87/handlebars-rust)

## Quick Start

Developed and tested with handlebars-rust v4.4.0.

### Registration

```rust
    use handlebars::Handlebars;
    use handlebars_concat::HandlebarsConcat;
    
    let mut h = Handlebars::new();
    h.register_helper("concat", Box::new(HandlebarsConcat));
```

### Usage

The helper is looking for multiple arguments of type string, array or object.
- String arguments are added directly to the output buffer.
- Array arguments are iterated and added as individual strings to the output buffer.
- Object arguments could be handled two different ways:
1. By default only the object keys are being used and the values are ignored.
2. If you add a block template the helper will use it to render the object value and  
concatenate it as string to the output buffer.

The helper accepts several hash arguments to modify the concatenation behavior:
- separator: Set specific string to join elements with. Default is ","
- distinct: Eliminate duplicates upon adding to output buffer
- quotes: Wrap each value in double quotation marks

Example with string literals:

```handlebars
{{concat "One" "Two" separator=", "}}
```

```handlebars
{{concat "One" "Two" separator=", " quotes=true}}
```

Where `s` is `"One"`, `arr` is `["One", "Two"]` and `obj` is `{"Three":3}`

```handlebars
{{concat s arr obj separator=", " distinct=true}}
```

Result: One, Two, Three

Where `s` is `"One"`, `arr` is `["One", "Two"]` and `obj` is `{"key0":{"label":"Two"},"key1":{"label":"Three"},"key2":{"label":"Four"}}`:

```handlebars
{{#concat s arr obj separator=", " distinct=true}}{{label}}{{/concat}}
```

Result: One, Two, Three, Four

## License

This library (handlebars-concat) is open sourced under the BSD 2 License.  
