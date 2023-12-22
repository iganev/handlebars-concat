use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, JsonRender, Output, RenderContext,
    Renderable, StringOutput,
};

const QUOTES_DOUBLE: &str = "\"";
const QUOTES_SINGLE: &str = "\'";

#[derive(Clone, Copy)]
/// Inflector helper for handlebars-rust
///
/// # Registration
///
/// ```
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
/// ```
///
/// # Arguments
///
/// Multiple mixed-type arguments are accepted. Any combination of types in any possible order is
/// allowed.
///
/// * String arguments are concatenated without any additional operations (except the optional quotation)
/// * Array arguments are iterated and each array value is treated as a separate string argument
/// * Object arguments have only their keys used for concatenation by default. If the helper is used
/// with a template block then objects values are being rendered using the template block before
/// being treated as strings and concatenated to the rest of the output.
///
/// # Hash parameters
///
/// * separator: Set specific string to join elements with. Default is ","
/// * distinct: Eliminate duplicates upon adding to output buffer
/// * quotes: Wrap each value in double quotation marks
/// * single_quote: Modifier of `quotes` to switch to single quotation mark instead
///
/// # Example usage:
///
/// * Using literals:
///
/// `
/// {{concat "One" "Two" separator=", "}}
/// `
///
/// Result: One, Two
///
///
/// `
/// {{concat "One" "Two" separator=", " quotes=true}}
/// `
///
/// Result: "One", "Two"
///
///
/// `
/// {{concat "One" "Two" separator=", " quotes=true single_quote=true}}
/// `
///
/// Result: 'One', 'Two'
///
///
/// * Where `s` is `"One"`, `arr` is `["One", "Two"]` and `obj` is `{"Three":3}`
///
/// `
/// {{concat s arr obj separator=", " distinct=true}}
/// `
///
/// Result: One, Two, Three
///
///
/// * Where `s` is `"One"`, `arr` is `["One", "Two"]` and `obj` is `{"key0":{"label":"Two"},"key1":{"label":"Three"},"key2":{"label":"Four"}}`
///
/// `
/// {{#concat s arr obj separator=", " distinct=true}}{{label}}{{/concat}}
/// `
///
/// Result: One, Two, Three, Four
///
pub struct HandlebarsConcat;

impl HelperDef for HandlebarsConcat {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars,
        _ctx: &'rc Context,
        _rc: &mut RenderContext<'reg, 'rc>,
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

        let template = h.template();

        let mut output: Vec<String> = Vec::new();

        for param in h.params() {
            let param = param.clone();

            if param.value().is_string() {
                let mut value = param.value().render();

                if quotes {
                    value = format!("{}{}{}", quotation_mark, value, quotation_mark);
                }

                if !output.contains(&value) || !distinct {
                    output.push(value);
                }
            } else if param.value().is_array() {
                if let Some(ar) = param.value().as_array() {
                    output.append(
                        &mut ar
                            .iter()
                            .map(|item| item.render())
                            .map(|item| {
                                if quotes {
                                    format!("{}{}{}", quotation_mark, item, quotation_mark)
                                } else {
                                    item
                                }
                            })
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
            } else if param.value().is_object() {
                if let Some(o) = param.value().as_object() {
                    if h.is_block() {
                        // use block template to render objects

                        for obj in o.values() {
                            let mut content = StringOutput::default();

                            let context_data = obj.clone();
                            let context = Context::from(context_data);
                            let mut render_context = RenderContext::new(None);

                            template
                                .map(|t| t.render(r, &context, &mut render_context, &mut content))
                                .unwrap_or(Ok(()))?;

                            if let Ok(out) = content.into_string() {
                                let result = if quotes {
                                    format!("{}{}{}", quotation_mark, out, quotation_mark)
                                } else {
                                    out
                                };

                                if !result.is_empty() && (!output.contains(&result) || !distinct) {
                                    output.push(result);
                                }
                            }
                        }
                    } else {
                        // render keys only

                        output.append(
                            &mut o
                                .keys()
                                .cloned()
                                .map(|item| {
                                    if quotes {
                                        format!("{}{}{}", quotation_mark, item, quotation_mark)
                                    } else {
                                        item
                                    }
                                })
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
    }
}
