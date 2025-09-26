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

fn transform(expr: &SExpr, in_rule_body: bool) -> SExpr {
    use SExpr::*;
    match expr {
        Atom(a) => Atom(a.clone()),
        List(items) => {
            // Special-case rule structure: (rule (query ...) (body ...) ...)
            if let Some(Atom(h)) = items.first() {
                if h == "rule" && items.len() >= 3 {
                    let mut out: Vec<SExpr> = Vec::with_capacity(items.len());
                    out.push(Atom("rule".to_string()));
                    // Query: no replacements
                    out.push(transform(&items[1], false));
                    // Body: enable replacements
                    out.push(transform(&items[2], true));
                    // Any trailing attrs or metadata: no replacements
                    for extra in items.iter().skip(3) {
                        out.push(transform(extra, false));
                    }
                    return List(out);
                }
            }

            // If inside a rule body, replace context constructors with DUMMYCTX
            if in_rule_body {
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
                        return Atom("DUMMYCTX".to_string());
                    }
                }
            }

            List(items.iter().map(|e| transform(e, in_rule_body)).collect())
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
    let transformed: Vec<SExpr> = parsed.iter().map(|e| transform(e, false)).collect();

    let res = transformed
        .iter()
        .map(|e| print_sexpr(e))
        .collect::<Vec<_>>()
        .join("\n");
    eprintln!("res: {}", res);
    res
}
