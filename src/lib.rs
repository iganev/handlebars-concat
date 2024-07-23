use handlebars::{
    BlockContext, Context, Handlebars, Helper, HelperDef, HelperResult, JsonRender, Output,
    PathAndJson, RenderContext, Renderable, ScopedJson, StringOutput,
};

const QUOTES_DOUBLE: &str = "\"";
const QUOTES_SINGLE: &str = "\'";

#[allow(clippy::assigning_clones)]
pub(crate) fn create_block<'rc>(param: &PathAndJson<'rc>) -> BlockContext<'rc> {
    let mut block = BlockContext::new();

    if let Some(new_path) = param.context_path() {
        *block.base_path_mut() = new_path.clone();
    } else {
        // use clone for now
        block.set_base_value(param.value().clone());
    }

    block
}

pub(crate) fn apply_wrapper(subject: String, wrapper: &str, wrap: bool) -> String {
    if wrap {
        format!("{}{}{}", wrapper, subject, wrapper)
    } else {
        subject
    }
}

#[derive(Clone, Copy)]
/// Concat helper for handlebars-rust
///
/// # Registration
///
/// ```rust
/// use handlebars::Handlebars;
/// use handlebars_concat::HandlebarsConcat;
/// use serde_json::json;
///
/// let mut h = Handlebars::new();
/// h.register_helper("concat", Box::new(HandlebarsConcat));
///
/// assert_eq!(h.render_template(r#"{{concat item1 item2}}"#, &json!({"item1": "Value 1", "item2": "Value 2"})).expect("Render error"), "Value 1,Value 2");
/// assert_eq!(h.render_template(r#"{{concat this separator=", "}}"#, &json!({"item1": "Value 1", "item2": "Value 2"})).expect("Render error"), "item1, item2");
/// assert_eq!(h.render_template(r#"{{#concat this separator=", "}}{{this}}{{/concat}}"#, &json!({"item1": "Value 1", "item2": "Value 2"})).expect("Render error"), "Value 1, Value 2");
/// assert_eq!(h.render_template(r#"{{#concat "Form" this separator="" render_all=true}}<{{#if tag}}{{tag}}{{else}}{{this}}{{/if}}/>{{/concat}}"#, &json!({"key0":{"tag":"Input"},"key1":{"tag":"Select"},"key2":{"tag":"Button"}})).expect("Render error"), "<Form/><Input/><Select/><Button/>");
/// ```
///
/// # Behavior
///
/// The helper is looking for multiple arguments of type string, array or object. Arguments are being added to an output buffer and returned altogether as string.
///
/// The helper has few parameters modifying the behavior slightly. For example `distinct=true` eliminates duplicate values from the output buffer, while `quotes=true` in combination with `single_quote=true` wraps the values in quotation marks.
///
/// ## String
/// ~~String arguments are added directly to the output buffer.~~
/// As of `0.1.3` strings could be handled in one of two ways:
/// 1. By default strings are added to the output buffer without modification (other than the quotation mark modifiers).
/// 2. If you add a block template and use the `render_all` parameter, strings will be passed as `{{this}}` to the block template.
///
/// The block template rendering is disabled by default for backward compatibility.
///
/// ## Array
/// ~~Array arguments are iterated and added as individual strings to the output buffer.~~
/// As of `0.1.3` arrays could be handled in one of two ways:
/// 1. By default array values are added as individual strings to the output buffer without modification (other than the quotation mark modifiers).
/// 2. If you add a block template and use the `render_all` parameter, array values are passed as `{{this}}` to the block template.
///
/// The block template rendering is disabled by default for backward compatibility.
///
/// ## Object
/// Object arguments could be handled two different ways:
/// 1. By default only the object keys are being used and the values are ignored.
/// 2. If you add a block template the helper will use it to render the object value and
/// concatenate it as string to the output buffer.
///
/// Object rendering results are subject to `distinct`, `quotes` and `single_quote` modifier parameters, just like strings and arrays.
///
/// # Hash parameters
///
/// * separator: Set specific string to join elements with. Default is ","
/// * distinct: Eliminate duplicates upon adding to output buffer
/// * quotes: Wrap each value in double quotation marks
/// * single_quote: Modifier of `quotes` to switch to single quotation mark instead
/// * render_all: Render all values using the block template, not just object values
///
/// # Example usage:
///
///
/// Example with string literals:
///
/// ```handlebars
/// {{concat "One" "Two" separator=", "}}
/// ```
///
/// Result: `One, Two`
///
/// ---
///
/// ```handlebars
/// {{concat "One" "Two" separator=", " quotes=true}}
/// ```
///
/// Result: `"One", "Two"`
///
/// ---
///
/// Where `s` is `"One"`, `arr` is `["One", "Two"]` and `obj` is `{"Three":3}`
///
/// ```handlebars
/// {{concat s arr obj separator=", " distinct=true}}
/// ```
///
/// Result: `One, Two, Three`
///
/// ---
///
/// Where `s` is `"One"`, `arr` is `["One", "Two"]` and `obj` is `{"key0":{"label":"Two"},"key1":{"label":"Three"},"key2":{"label":"Four"}}`:
///
/// ```handlebars
/// {{#concat s arr obj separator=", " distinct=true}}{{label}}{{/concat}}
/// ```
///
/// Result: `One, Two, Three, Four`
///
/// ---
///
/// Where `s` is `"One"`, `arr` is `["One", "Two"]` and `obj` is `{"key0":{"label":"Two"},"key1":{"label":"Three"},"key2":{"label":"Four"}}`
///
/// ```handlebars
/// {{#concat s arr obj separator=", " distinct=true render_all=true}}<{{#if label}}{{label}}{{else}}{{this}}{{/if}}/>{{/concat}}
/// ```
///
/// Result: `<One/>, <Two/>, <Three/>, <Four/>`
///
/// ---
///
pub struct HandlebarsConcat;

impl HelperDef for HandlebarsConcat {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        r: &'reg Handlebars,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let separator = if let Some(s) = h.hash_get("separator") {
            s.render()
        } else {
            ",".to_string()
        };

        let distinct = h.hash_get("distinct").is_some();

        let quotes = h.hash_get("quotes").is_some();
        let single_quote = h.hash_get("single_quote").is_some(); // as a modifier on top of "quotes", switches to single quotation

        let quotation_mark = if quotes {
            if single_quote {
                QUOTES_SINGLE
            } else {
                QUOTES_DOUBLE
            }
        } else {
            ""
        };

        let render_all = h.hash_get("render_all").is_some(); // force all values through the block template

        let template = h.template();

        let mut output: Vec<String> = Vec::new();

        for param in h.params() {
            match param.value() {
                serde_json::Value::Null => {}
                serde_json::Value::Bool(_)
                | serde_json::Value::Number(_)
                | serde_json::Value::String(_) => {
                    let value = if h.is_block() && render_all {
                        // use block template to render strings

                        let mut content = StringOutput::default();

                        rc.push_block(create_block(&param));
                        template
                            .map(|t| t.render(r, ctx, rc, &mut content))
                            .unwrap_or(Ok(()))?;
                        rc.pop_block();

                        content.into_string().unwrap_or_default()
                    } else {
                        param.value().render()
                    };

                    let value = apply_wrapper(value, quotation_mark, quotes);

                    if !value.is_empty() && (!output.contains(&value) || !distinct) {
                        output.push(value);
                    }
                }
                serde_json::Value::Array(ar) => {
                    if h.is_block() && render_all {
                        // use block template to render array elements

                        for array_item in ar {
                            let mut content = StringOutput::default();

                            let block = create_block(&PathAndJson::new(
                                None,
                                ScopedJson::from(array_item.clone()),
                            ));
                            rc.push_block(block);

                            template
                                .map(|t| t.render(r, ctx, rc, &mut content))
                                .unwrap_or(Ok(()))?;

                            rc.pop_block();

                            if let Ok(value) = content.into_string() {
                                let value = apply_wrapper(value, quotation_mark, quotes);

                                if !value.is_empty() && (!output.contains(&value) || !distinct) {
                                    output.push(value);
                                }
                            }
                        }
                    } else {
                        output.append(
                            &mut ar
                                .iter()
                                .map(|item| item.render())
                                .map(|item| apply_wrapper(item, quotation_mark, quotes))
                                .filter(|item| {
                                    if distinct {
                                        !output.contains(item)
                                    } else {
                                        true
                                    }
                                })
                                .collect::<Vec<String>>(),
                        );
                    }
                }
                serde_json::Value::Object(o) => {
                    if h.is_block() {
                        // use block template to render objects

                        for obj in o.values() {
                            let mut content = StringOutput::default();

                            let block = create_block(&PathAndJson::new(
                                None,
                                ScopedJson::from(obj.clone()),
                            ));
                            rc.push_block(block);

                            template
                                .map(|t| t.render(r, ctx, rc, &mut content))
                                .unwrap_or(Ok(()))?;

                            rc.pop_block();

                            if let Ok(value) = content.into_string() {
                                let value = apply_wrapper(value, quotation_mark, quotes);

                                if !value.is_empty() && (!output.contains(&value) || !distinct) {
                                    output.push(value);
                                }
                            }
                        }
                    } else {
                        // render keys only

                        output.append(
                            &mut o
                                .keys()
                                .cloned()
                                .map(|item| apply_wrapper(item, quotation_mark, quotes))
                                .filter(|item| {
                                    if distinct {
                                        !output.contains(item)
                                    } else {
                                        true
                                    }
                                })
                                .collect::<Vec<String>>(),
                        );
                    }
                }
            }
        }

        out.write(&output.join(&*separator))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        use handlebars::Handlebars;
        use serde_json::json;

        let mut h = Handlebars::new();
        h.register_helper("concat", Box::new(HandlebarsConcat));

        assert_eq!(
            h.render_template(r#"{{concat 1 2}}"#, &String::new())
                .expect("Render error"),
            "1,2",
            "Failed to concat numeric literals"
        );
        assert_eq!(
            h.render_template(r#"{{concat "One" "Two"}}"#, &String::new())
                .expect("Render error"),
            "One,Two",
            "Failed to concat literals"
        );
        assert_eq!(
            h.render_template(r#"{{concat "One" "Two" separator=", "}}"#, &String::new())
                .expect("Render error"),
            "One, Two",
            "Failed to concat literals with separator"
        );
        assert_eq!(
            h.render_template(
                r#"{{concat "One" "Two" separator=", " quotes=true}}"#,
                &String::new()
            )
            .expect("Render error"),
            r#""One", "Two""#,
            "Failed to concat literals with separator and quotes"
        );
        assert_eq!(
            h.render_template(
                r#"{{concat "One" "Two" separator=", " quotes=true single_quote=true}}"#,
                &String::new()
            )
            .expect("Render error"),
            r#"'One', 'Two'"#,
            "Failed to concat literals with separator and single quotation marks"
        );
        assert_eq!(
            h.render_template(
                r#"{{concat s arr obj separator=", " quotes=true}}"#,
                &json!({"arr": ["One", "Two", "Three"]})
            )
            .expect("Render error"),
            r#""One", "Two", "Three""#,
            "Failed to concat array with quotes"
        );
        assert_eq!(
            h.render_template(
                r#"{{concat s arr obj separator=", " distinct=true}}"#,
                &json!({"s": "One", "arr": ["One", "Two"], "obj": {"Three":3}})
            )
            .expect("Render error"),
            "One, Two, Three",
            "Failed to concat literal, array and object"
        );
        assert_eq!(
            h.render_template(
                r#"{{#concat s arr obj separator=", " distinct=true}}{{label}}{{/concat}}"#,
                &json!({"s": "One", "arr": ["One", "Two"], "obj": {"key0":{"label":"Two"},"key1":{"label":"Three"},"key2":{"label":"Four"}}})
            ).expect("Render error"),
            "One, Two, Three, Four",
            "Failed to concat literal, array and object using block template"
        );
        assert_eq!(
            h.render_template(
                r#"{{#concat s arr obj separator=", " distinct=true quotes=true}}{{label}}{{/concat}}"#,
                &json!({"s": "One", "arr": ["One", "Two"], "obj": {"key0":{"label":"Two"},"key1":{"label":"Three"},"key2":{"label":"Four"}}})
            ).expect("Render error"),
            r#""One", "Two", "Three", "Four""#,
            "Failed to concat literal, array and object using block template"
        );
        assert_eq!(
            h.render_template(
                r#"{{concat obj separator=", " quotes=true}}"#,
                &json!({"obj": {"key0":{"label":"Two"},"key1":{"label":"Three"},"key2":{"label":"Four"}}})
            ).expect("Render error"),
            r#""key0", "key1", "key2""#,
            "Failed to concat object keys with quotation marks and no distinction"
        );
        assert_eq!(
            h.render_template(
                r#"{{#concat s arr obj separator=", " distinct=true render_all=true}}<{{#if label}}{{label}}{{else}}{{this}}{{/if}}/>{{/concat}}"#,
                &json!({"s": "One", "arr": ["One", "Two"], "obj": {"key0":{"label":"Two"},"key1":{"label":"Three"},"key2":{"label":"Four"}}})
            ).expect("Render error"),
            r#"<One/>, <Two/>, <Three/>, <Four/>"#,
            "Failed to concat literal, array and object using block template"
        );
        assert_eq!(
            h.render_template(
                r#"{{#concat s arr obj separator=", " distinct=true render_all=true quotes=true}}[{{#if label}}{{label}}{{else}}{{this}}{{/if}}]{{/concat}}"#,
                &json!({"s": "One", "arr": ["One", "Two"], "obj": {"key0":{"label":"Two"},"key1":{"label":"Three"},"key2":{"label":"Four"}}})
            ).expect("Render error"),
            r#""[One]", "[Two]", "[Three]", "[Four]""#,
            "Failed to concat literal, array and object using block template"
        );
        assert_eq!(
            h.render_template(
                r#"{{#concat s arr obj separator=", " distinct=true render_all=true quotes=true}}[{{#if label}}{{label}}{{else}}{{@root/zero}}{{/if}}]{{/concat}}"#,
                &json!({"zero":"Zero", "s": "One", "arr": ["One", "Two"], "obj": {"key0":{"label":"Two"},"key1":{"label":"Three"},"key2":{"label":"Four"}}})
            ).expect("Render error"),
            r#""[Zero]", "[Two]", "[Three]", "[Four]""#,
            "Failed to concat literal, array and object using block template"
        );
    }
}
