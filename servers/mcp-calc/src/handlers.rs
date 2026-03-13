use crate::error::AgentError;
use crate::state::AppState;
use serde_json::Value;
use tracing::{debug, error, info};

// TODO: implement tools in this file
use crate::state::MathState;
use mathexpr::Expression;

pub fn compute(params: Option<Value>, state: &mut MathState) -> Value {
    debug!(" payload: {:?}", params);
    let result = (|| -> Result<f64, AgentError> {
        // Utilise bien AgentError
        let p = params
            .as_ref()
            .ok_or_else(|| AgentError::Parse("missing params".into()))?;

        // 1. Extraction flexible de l'expression
        let expr_str = p["expression"]
            .as_str()
            .or_else(|| p["arguments"]["expression"].as_str()) // Fallback si encapsulé
            .ok_or_else(|| AgentError::Parse("missing 'expression' field".into()))?;

        if expr_str.trim().is_empty() {
            return Err(AgentError::Parse("expression is empty".into()));
        }

        // 2. Extraction flexible des variables (vars)
        let vars_map = p["vars"]
            .as_object()
            .or_else(|| p["arguments"]["vars"].as_object());

        let mut pairs: Vec<(String, f64)> = match vars_map {
            Some(map) => map
                .iter()
                .filter_map(|(k, v)| v.as_f64().map(|f| (k.clone(), f)))
                .collect(),
            None => vec![],
        };
        pairs.sort_by(|a, b| a.0.cmp(&b.0));

        let (var_names, var_values): (Vec<String>, Vec<f64>) = pairs.into_iter().unzip();
        let var_refs: Vec<&str> = var_names.iter().map(|s| s.as_str()).collect();
        let cache_key = format!("{}|{}", expr_str, var_names.join(","));

        debug!(expr = %expr_str, vars = ?var_names, "Evaluating expression");

        if !state.cache.contains_key(&cache_key) {
            info!(expr = %expr_str, "Compiling new expression");
            let compiled = Expression::parse(expr_str)
                .map_err(|e| AgentError::Parse(e.to_string()))?
                .compile(&var_refs)
                .map_err(|e| AgentError::Compile(e.to_string()))?;
            state.cache.insert(cache_key.clone(), compiled);
        } else {
            debug!(expr = %expr_str, "Using cached compiled expression");
        }

        state.cache[&cache_key]
            .eval(&var_values)
            .map_err(|e| AgentError::Eval(e.to_string()))
    })();

    match result {
        Ok(val) => {
            info!(result = %val, "Evaluation successful");
            serde_json::json!({
                "content": [{ "type": "text", "text": val.to_string() }],
                "isError": false
            })
        }
        Err(e) => {
            error!(error = %e, "Evaluation failed");
            serde_json::json!({
            "isError": true,
            "content": [{ "type": "text", "text": error_guidance(&e) }]
            })
        }
    }
}

fn error_guidance(e: &AgentError) -> String {
    match e {
        AgentError::Parse(msg) => format!(
            "Expression syntax error: {}. \
            Check operator notation (use * not × or x, use ^ for powers), \
            balanced parentheses, and that function names are lowercase (sqrt, sin, log...). \
            Example of valid expression: 'sqrt(x^2 + y^2)'.",
            msg
        ),
        AgentError::Compile(msg) => format!(
            "Unknown variable or function: {}. \
            If your expression contains variables (x, r, n...), \
            make sure to declare them in the 'vars' field. \
            Built-in constants 'pi' and 'e' do not need to be declared. \
            Example: expression='2*pi*r', vars={{\"r\": 6371.0}}.",
            msg
        ),
        AgentError::Eval(msg) => format!(
            "Evaluation error: {}. \
            Common causes: division by zero, sqrt of a negative number, log of zero or negative. \
            Check your variable values in the 'vars' field.",
            msg
        ),
        _ => format!("Internal error: {}. Please retry.", e),
    }
}
