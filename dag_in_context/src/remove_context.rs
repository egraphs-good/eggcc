#[derive(Clone, Debug, PartialEq, Eq)]
enum SExpr {
    Atom(String),
    List(Vec<SExpr>),
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut i = 0;
    let bytes = input.as_bytes();
    while i < bytes.len() {
        let c = bytes[i] as char;
        // Skip whitespace
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        // Skip line comments starting with ';'
        if c == ';' {
            while i < bytes.len() && (bytes[i] as char) != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '(' || c == ')' {
            tokens.push(c.to_string());
            i += 1;
            continue;
        }
        if c == '"' {
            // string literal, keep quotes
            let start = i;
            i += 1;
            while i < bytes.len() {
                let ch = bytes[i] as char;
                if ch == '\\' {
                    // skip escaped next char if present
                    i += 2;
                    continue;
                }
                if ch == '"' {
                    i += 1; // include closing quote
                    break;
                }
                i += 1;
            }
            tokens.push(input[start..i].to_string());
            continue;
        }
        // symbol/number token
        let start = i;
        while i < bytes.len() {
            let ch = bytes[i] as char;
            if ch.is_whitespace() || ch == '(' || ch == ')' || ch == ';' {
                break;
            }
            i += 1;
        }
        tokens.push(input[start..i].to_string());
    }
    tokens
}

fn parse(tokens: &[String]) -> Vec<SExpr> {
    let mut stack: Vec<Vec<SExpr>> = vec![Vec::new()];
    let mut i = 0;
    while i < tokens.len() {
        match tokens[i].as_str() {
            "(" => stack.push(Vec::new()),
            ")" => {
                let list = stack.pop().expect("Unbalanced parentheses: too many )");
                stack
                    .last_mut()
                    .expect("Unbalanced parentheses: empty stack")
                    .push(SExpr::List(list));
            }
            atom => stack
                .last_mut()
                .expect("Parser stack empty")
                .push(SExpr::Atom(atom.to_string())),
        }
        i += 1;
    }
    if stack.len() != 1 {
        // Unbalanced parentheses; return input as a single atom to avoid crash
        return vec![SExpr::Atom(tokens.join(" "))];
    }
    stack.pop().unwrap()
}

fn sexpr_head(list: &SExpr) -> Option<&str> {
    if let SExpr::List(items) = list {
        if let Some(SExpr::Atom(h)) = items.first() {
            return Some(h.as_str());
        }
    }
    None
}

fn contains_context(expr: &SExpr) -> bool {
    match expr {
        SExpr::Atom(_) => false,
        SExpr::List(items) => {
            if let Some(head) = sexpr_head(expr) {
                let arity = items.len().saturating_sub(1);
                let is_ctx = match head {
                    "InFunc" => arity == 1,
                    "InLoop" => arity == 2,
                    "InSwitch" => arity == 3,
                    "InIf" => arity == 3,
                    _ => false,
                };
                if is_ctx {
                    return true;
                }
            }
            items.iter().any(contains_context)
        }
    }
}

fn replace_context_with_dummy(expr: &SExpr) -> SExpr {
    match expr {
        SExpr::Atom(a) => SExpr::Atom(a.clone()),
        SExpr::List(items) => {
            if let Some(head) = sexpr_head(expr) {
                let arity = items.len().saturating_sub(1);
                let is_ctx = matches!(head, "InFunc" | "InLoop" | "InSwitch" | "InIf")
                    && ((head == "InFunc" && arity == 1)
                        || (head == "InLoop" && arity == 2)
                        || (head == "InSwitch" && arity == 3)
                        || (head == "InIf" && arity == 3));
                if is_ctx {
                    return SExpr::Atom("DUMMYCTX".to_string());
                }
            }
            SExpr::List(items.iter().map(replace_context_with_dummy).collect())
        }
    }
}

fn transform(expr: &SExpr) -> SExpr {
    use SExpr::*;
    match expr {
        Atom(a) => Atom(a.clone()),
        List(items) => {
            // Special-case rule structure: (rule (query ...) (body ...) ...)
            if let Some(Atom(h)) = items.first() {
                if h == "rule" && items.len() >= 3 {
                    let mut out: Vec<SExpr> = Vec::with_capacity(items.len());
                    out.push(Atom("rule".to_string()));
                    // Query: recurse without replacement to normalize
                    let new_query = transform(&items[1]);
                    let query_has_ctx = contains_context(&new_query);
                    out.push(new_query);
                    // Body handling depends on whether query references context
                    let body = &items[2];
                    let new_body = if query_has_ctx {
                        if contains_context(body) {
                            List(vec![List(vec![
                                Atom("panic".to_string()),
                                Atom("\"context should not be present\"".to_string()),
                            ])])
                        } else {
                            transform(body)
                        }
                    } else {
                        // Query doesn't use context: replace contexts in body with DUMMYCTX
                        replace_context_with_dummy(body)
                    };
                    out.push(new_body);
                    // Any trailing attrs or metadata
                    for extra in items.iter().skip(3) {
                        out.push(transform(extra));
                    }
                    return List(out);
                }
            }

            List(items.iter().map(transform).collect())
        }
    }
}

fn print_sexpr(expr: &SExpr) -> String {
    match expr {
        SExpr::Atom(a) => a.clone(),
        SExpr::List(items) => {
            let inner = items.iter().map(print_sexpr).collect::<Vec<_>>().join(" ");
            format!("({})", inner)
        }
    }
}

pub(crate) fn remove_new_contexts(s: &str) -> String {
    let tokens = tokenize(s);
    let parsed = parse(&tokens);
    let transformed: Vec<SExpr> = parsed.iter().map(transform).collect();
    transformed
        .iter()
        .map(print_sexpr)
        .collect::<Vec<_>>()
        .join("\n")
}
