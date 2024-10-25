use nu_engine::command_prelude::*;

#[derive(Clone)]
pub struct SubCommand;

impl Command for SubCommand {
    fn name(&self) -> &str {
        "url build-query"
    }

    fn signature(&self) -> Signature {
        Signature::build("url build-query")
            .input_output_types(vec![
                (Type::record(), Type::String),
                (Type::table(), Type::String),
            ])
            .category(Category::Network)
    }

    fn description(&self) -> &str {
        "Converts record or table into query string applying percent-encoding."
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["convert", "record", "table"]
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Outputs a query string representing the contents of this record",
                example: r#"{ mode:normal userid:31415 } | url build-query"#,
                result: Some(Value::test_string("mode=normal&userid=31415")),
            },
            Example {
                description: "Outputs a query string representing the contents of this 1-row table",
                example: r#"[[foo bar]; ["1" "2"]] | url build-query"#,
                result: Some(Value::test_string("foo=1&bar=2")),
            },
            Example {
                description: "Outputs a query string representing the contents of this record",
                example: r#"{a:"AT&T", b: "AT T"} | url build-query"#,
                result: Some(Value::test_string("a=AT%26T&b=AT+T")),
            },
            Example {
                description: "Outputs a query string representing the contents of this record",
                example: r#"{a: ["one", "two"], b: "three"} | url build-query"#,
                result: Some(Value::test_string("a=one&a=two&b=three")),
            },
        ]
    }

    fn run(
        &self,
        _engine_state: &EngineState,
        _stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let head = call.head;
        to_url(input, head)
    }
}

fn to_url(input: PipelineData, head: Span) -> Result<PipelineData, ShellError> {
    let output: Result<String, ShellError> = input
        .into_iter()
        .map(move |value| {
            let span = value.span();
            match value {
                Value::Record { ref val, .. } => {
                    let mut row_vec = vec![];
                    for (k, v) in &**val {
                        match v {
                            Value::List { ref vals, .. } => {
                                for v_item in vals {
                                    row_vec.push((
                                        k.clone(),
                                        v_item.coerce_string().map_err(|_| {
                                            ShellError::UnsupportedInput {
                                                msg: "Expected a record with list of string values"
                                                    .to_string(),
                                                input: "value originates from here".into(),
                                                msg_span: head,
                                                input_span: span,
                                            }
                                        })?,
                                    ));
                                }
                            }
                            _ => row_vec.push((
                                k.clone(),
                                v.coerce_string()
                                    .map_err(|_| ShellError::UnsupportedInput {
                                        msg:
                                            "Expected a record with string or list of string values"
                                                .to_string(),
                                        input: "value originates from here".into(),
                                        msg_span: head,
                                        input_span: span,
                                    })?,
                            )),
                        }
                    }

                    serde_urlencoded::to_string(row_vec).map_err(|_| ShellError::CantConvert {
                        to_type: "URL".into(),
                        from_type: value.get_type().to_string(),
                        span: head,
                        help: None,
                    })
                }
                // Propagate existing errors
                Value::Error { error, .. } => Err(*error),
                other => Err(ShellError::UnsupportedInput {
                    msg: "Expected a table from pipeline".to_string(),
                    input: "value originates from here".into(),
                    msg_span: head,
                    input_span: other.span(),
                }),
            }
        })
        .collect();

    Ok(Value::string(output?, head).into_pipeline_data())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(SubCommand {})
    }
}
