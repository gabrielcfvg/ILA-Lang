
// local
use crate::ast::*;

// external
use anyhow::Result;


/* -------------------------------------------------------------------------- */
/*                                    utils                                   */
/* -------------------------------------------------------------------------- */

fn parse_node_lexical_info(node: &tree_sitter::Node) -> LexicalInfo {

    let start_point = node.start_position();
    let end_point = node.end_position();

    let start_line = start_point.row;
    let start_column = start_point.column;
    let end_line = end_point.row;
    let end_column = end_point.column;

    let start = LexicalPosition{line: start_line, column: start_column};
    let end = LexicalPosition{line: end_line, column: end_column};

    return LexicalInfo{start, end};
}

fn check_invalid_node(node: &tree_sitter::Node) -> Result<()> {

    if node.is_error() || node.is_missing() {

        let node_start = node.start_position();
        let node_end = node.end_position();
        return Err(anyhow::anyhow!("Invalid node, start: ({}, {}), end: ({}, {})",
            node_start.row, node_start.column, node_end.row, node_end.column));
    }
    
    return Ok(());
}

fn for_every_child_with_field_name(node: &tree_sitter::Node, field_name: &str, mut f: impl FnMut(&tree_sitter::Node) -> Result<()>) -> Result<()> {

    for child in 0..(node.child_count()) {

        if let Some(child_field_name) = node.field_name_for_child(child as u32) {
        
            if child_field_name != field_name {
                
                continue;
            }

            let child_node = node.child(child).unwrap();
            f(&child_node)?;
        }
    }

    return Ok(());
}


/* -------------------------------------------------------------------------- */
/*                              parsing functions                             */
/* -------------------------------------------------------------------------- */

fn parse_identifier(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<IdentifierID> {
    
    assert_eq!(node.kind(), "identifier");
    check_invalid_node(node)?;

    let name = node.utf8_text(state.source).unwrap().to_string();
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_identifier_node(name, lexical_info);
    return Ok(id);
}


fn parse_raw_type(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<TypeExprID> {

    assert_eq!(node.kind(), "raw_type");
    check_invalid_node(node)?;

    let name_field = node.child_by_field_name("name").expect("missing obligatory field");
    let name = parse_identifier(&name_field, state)?;

    let type_expr = TypeExpr::RawType{type_name: name};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_type_expr_node(type_expr, lexical_info);
    return Ok(id);
}

fn parse_template_type(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<TypeExprID> {

    assert_eq!(node.kind(), "template_type");
    check_invalid_node(node)?;

    let name_field = node.child_by_field_name("name").expect("missing obligatory field");
    let name = parse_identifier(&name_field, state)?;

    let mut args = Vec::new();
    for child in 0..(node.child_count()) {

        let child_node = node.child(child).unwrap();

        if child_node.kind() != "arg" {
            
            continue;
        }

        let arg_id = parse_type_expr(&child_node, state)?;
        args.push(arg_id);
    }

    let type_expr = TypeExpr::TemplateType{type_name: name, type_args: args};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_type_expr_node(type_expr, lexical_info);
    return Ok(id);
}

fn parse_ref_type(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<TypeExprID> {

    assert_eq!(node.kind(), "ref_type");
    check_invalid_node(node)?;

    let is_mut = node.child_by_field_name("is_mut").is_some();

    let type_field = node.child_by_field_name("type").expect("missing obligatory field");
    let type_expr = parse_type_expr(&type_field, state)?;

    let type_expr = TypeExpr::RefType{is_mut, type_expr};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_type_expr_node(type_expr, lexical_info);
    return Ok(id);
}

fn parse_comp_type(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<TypeExprID> {

    assert_eq!(node.kind(), "comp_type");
    check_invalid_node(node)?;

    let is_mut = node.child_by_field_name("is_mut").is_some();

    let type_field = node.child_by_field_name("type").expect("missing obligatory field");
    let type_expr = parse_type_expr(&type_field, state)?;

    let type_expr = TypeExpr::CompType{is_mut, type_expr};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_type_expr_node(type_expr, lexical_info);
    return Ok(id);
}

fn parse_type_expr(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<TypeExprID> {

    let node_type = node.kind();

    match node_type {
        "raw_type" => {
            return parse_raw_type(node, state);
        },
        "template_type" => {
            return parse_template_type(node, state);
        },
        "ref_type" => {
            return parse_ref_type(node, state);
        },
        "comp_type" => {
            return parse_comp_type(node, state);
        },
        _ => {
            panic!("Unknown or unexpected node type: {}", node_type);
        }
    }
}

fn parse_binary_oprt_str(oprt: &str) -> BinaryOprt {

    match oprt {
        
        "=" => BinaryOprt::Assign,
        "e" => BinaryOprt::And,
        "ou" => BinaryOprt::Or,
        "==" => BinaryOprt::Eq,
        "!=" => BinaryOprt::Neq,
        "<" => BinaryOprt::Lt,
        ">" => BinaryOprt::Gt,
        "<=" => BinaryOprt::Le,
        ">=" => BinaryOprt::Ge,
        "+" => BinaryOprt::Add,
        "-" => BinaryOprt::Sub,
        "*" => BinaryOprt::Mul,
        "/" => BinaryOprt::Div,
        _ => panic!("Unexpected binary operator: {}", oprt),
    }
}

fn parse_binary_expr(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<ExprID> {

    assert_eq!(node.kind(), "binary_expr");
    check_invalid_node(node)?;

    let oprt_field = node.child_by_field_name("oprt").expect("missing obligatory field");
    let oprt_str = oprt_field.utf8_text(state.source).unwrap();
    let oprt = parse_binary_oprt_str(oprt_str);

    let lhs_field = node.child_by_field_name("lhs").expect("missing obligatory field");
    let lhs = parse_expr(&lhs_field, state)?;

    let rhs_field = node.child_by_field_name("rhs").expect("missing obligatory field");
    let rhs = parse_expr(&rhs_field, state)?;

    let expr = Expression::BinaryOprt{oprt, left: lhs, right: rhs};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_expression_node(expr, lexical_info);
    return Ok(id);
}

fn parse_unary_oprt_str(oprt: &str) -> UnaryOprt {

    match oprt {
        
        "*" => UnaryOprt::Deref,
        "não" => UnaryOprt::Not,
        "-" => UnaryOprt::Neg,    
        _ => panic!("Unexpected unary operator: {}", oprt),
    }
}

fn parse_unary_expr(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<ExprID> {

    assert_eq!(node.kind(), "unary_expr");
    check_invalid_node(node)?;

    let oprt_field = node.child_by_field_name("oprt").expect("missing obligatory field");
    let oprt_str = oprt_field.utf8_text(state.source).unwrap();
    let oprt = parse_unary_oprt_str(oprt_str);

    let value_field = node.child_by_field_name("value").expect("missing obligatory field");
    let value = parse_expr(&value_field, state)?;

    let expr = Expression::UnaryOprt{oprt, operand: value};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_expression_node(expr, lexical_info);
    return Ok(id);
}

fn parse_access_expr(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<ExprID> {

    assert_eq!(node.kind(), "access_expr");
    check_invalid_node(node)?;

    let object_field = node.child_by_field_name("object").expect("missing obligatory field");
    let object = parse_expr(&object_field, state)?;

    let item_field = node.child_by_field_name("item").expect("missing obligatory field");
    let item = parse_identifier(&item_field, state)?;

    let expr = Expression::Access{object: object, field_name: item};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_expression_node(expr, lexical_info);
    return Ok(id);
}

fn parse_call_expr(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<ExprID> {

    assert_eq!(node.kind(), "call_expr");
    check_invalid_node(node)?;

    let function_field = node.child_by_field_name("function").expect("missing obligatory field");
    let function = parse_expr(&function_field, state)?;

    let mut args = Vec::new();
    for_every_child_with_field_name(node, "arg", |child_node| {
        
        let arg_id = parse_expr(&child_node, state)?;
        args.push(arg_id);
        return Ok(());
    })?;

    let expr = Expression::Call{callee: function, args};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_expression_node(expr, lexical_info);
    return Ok(id);
}

fn parse_parem_expr(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<ExprID> {

    assert_eq!(node.kind(), "parem_expr");
    check_invalid_node(node)?;

    let expr_field = node.child_by_field_name("expression").expect("missing obligatory field");
    let expr = parse_expr(&expr_field, state)?;

    return Ok(expr);
}

fn parse_identifier_expr(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<ExprID> {

    assert_eq!(node.kind(), "identifier");
    check_invalid_node(node)?;

    let expr = parse_identifier(&node, state)?;
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_expression_node(Expression::Identifier{node_id: expr}, lexical_info);
    return Ok(id);
}

fn parse_decimal_literal(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<ExprID> {

    assert_eq!(node.kind(), "decimal");
    check_invalid_node(node)?;

    let integer_field = node.child_by_field_name("integer").expect("missing obligatory field");
    let integer = integer_field.utf8_text(state.source).unwrap().parse::<i64>().unwrap();

    let fraction_field = node.child_by_field_name("fraction").expect("missing obligatory field");
    let mut fraction = fraction_field.utf8_text(state.source).unwrap().parse::<i64>().unwrap();

    while fraction >= 1 {
        fraction /= 10;
    }

    let value = integer as f64 + fraction as f64;

    let expr = Expression::DecimalLiteral{value};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_expression_node(expr, lexical_info);
    return Ok(id);
}

fn parse_integer_literal(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<ExprID> {

    assert_eq!(node.kind(), "integer");
    check_invalid_node(node)?;

    let value_field = node.child_by_field_name("value").expect("missing obligatory field");
    let value = value_field.utf8_text(state.source).unwrap().parse::<i64>().unwrap();

    let expr = Expression::IntegerLiteral {value};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_expression_node(expr, lexical_info);
    return Ok(id);
}

fn parse_string_literal(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<ExprID> {

    assert_eq!(node.kind(), "string");
    check_invalid_node(node)?;

    let content_field = node.child_by_field_name("content").expect("missing obligatory field");
    let content = content_field.utf8_text(state.source).unwrap().to_string();

    let expr = Expression::StringLiteral{value: content};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_expression_node(expr, lexical_info);
    return Ok(id);
}

fn parse_bool_literal(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<ExprID> {

    assert_eq!(node.kind(), "boolean");
    check_invalid_node(node)?;

    let value_field = node.child_by_field_name("value").expect("missing obligatory field");
    let value = value_field.utf8_text(state.source).unwrap();

    let value = match value {
        "verdadeiro" => true,
        "falso" => false,
        _ => panic!("Unexpected boolean value: {}", value),
    };

    let expr = Expression::BooleanLiteral{value};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_expression_node(expr, lexical_info);
    return Ok(id);
}

fn parse_list_literal(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<ExprID> {

    assert_eq!(node.kind(), "list");
    check_invalid_node(node)?;

    let mut items = Vec::new();
    for_every_child_with_field_name(node, "item", |child_node| {
        
        let expr_id = parse_expr(&child_node, state)?;
        items.push(expr_id);
        return Ok(());
    })?;

    let expr = Expression::ListLiteral{values: items};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_expression_node(expr, lexical_info);
    return Ok(id);
}


fn parse_expr(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<ExprID> {

    let node_type = node.kind();

    match node_type {
        "binary_expr" => {
            return parse_binary_expr(node, state);
        },
        "unary_expr" => {
            return parse_unary_expr(node, state);
        },
        "access_expr" => {
            return parse_access_expr(node, state);
        },
        "call_expr" => {
            return parse_call_expr(node, state);
        },
        "parem_expr" => {
            return parse_parem_expr(node, state);
        },
        "identifier" => {
            return parse_identifier_expr(node, state);
        },
        "decimal" => {
            return parse_decimal_literal(node, state);
        },
        "integer" => {
            return parse_integer_literal(node, state);
        },
        "string" => {
            return parse_string_literal(node, state);
        },
        "boolean" => {
            return parse_bool_literal(node, state);
        },
        "list" => {
            return parse_list_literal(node, state);
        },
        _ => {
            panic!("Unknown or unexpected node type: {}", node_type);
        }
    }
}

fn parse_break_stmt(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<StmtID> {

    assert_eq!(node.kind(), "break_stmt");
    check_invalid_node(node)?;

    let stmt = Statement::Break;
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_statement_node(stmt, lexical_info);
    return Ok(id);
}

fn parse_continue_stmt(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<StmtID> {

    assert_eq!(node.kind(), "continue_stmt");
    check_invalid_node(node)?;

    let stmt = Statement::Continue;
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_statement_node(stmt, lexical_info);
    return Ok(id);
}

fn parse_return_stmt(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<StmtID> {

    assert_eq!(node.kind(), "return_stmt");
    check_invalid_node(node)?;

    let mut return_expr = None;
    if let Some(expr_field) = node.child_by_field_name("return_expr") {
        return_expr = Some(parse_expr(&expr_field, state)?);
    }

    let stmt = Statement::Return{expr: return_expr};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_statement_node(stmt, lexical_info);
    return Ok(id);
}

fn for_each_item(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<ForEachDeclID> {

    assert_eq!(node.kind(), "for_item_decl");
    check_invalid_node(node)?;

    let is_mut = node.child_by_field_name("is_mut").is_some();
    let is_ref = node.child_by_field_name("is_ref").is_some();

    let name_field = node.child_by_field_name("name").expect("missing obligatory field");
    let name = parse_identifier(&name_field, state)?;

    let for_each_decl = ForEachDecl{is_mut, is_ref, name};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_for_each_decl_node(for_each_decl, lexical_info);
    return Ok(id);
}

fn parse_for_each_stmt(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<StmtID> {

    assert_eq!(node.kind(), "for_each_loop");
    check_invalid_node(node)?;

    let item_field = node.child_by_field_name("item").expect("missing obligatory field");
    let item_id = for_each_item(&item_field, state)?;

    let iterator_field = node.child_by_field_name("iterator").expect("missing obligatory field");
    let iterator = parse_expr(&iterator_field, state)?;

    let mut body = Vec::new();
    for_every_child_with_field_name(node, "body", |child_node| {
        
        let stmt_id = parse_stmt(&child_node, state)?;
        body.push(stmt_id);
        return Ok(());
    })?;

    let for_each_loop = Statement::ForEach{item: item_id, iter_expr: iterator, body_block: body};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_statement_node(for_each_loop, lexical_info);
    return Ok(id);
}

fn parse_while_stmt(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<StmtID> {

    assert_eq!(node.kind(), "while_loop");
    check_invalid_node(node)?;

    let condition_field = node.child_by_field_name("condition").expect("missing obligatory field");
    let condition = parse_expr(&condition_field, state)?;

    let mut body = Vec::new();
    for_every_child_with_field_name(node, "body", |child_node| {
        
        let stmt_id = parse_stmt(&child_node, state)?;
        body.push(stmt_id);
        return Ok(());
    })?;

    let while_loop = Statement::While{cond_expr: condition, body_block: body};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_statement_node(while_loop, lexical_info);
    return Ok(id);
}

fn parse_conditional_stmt(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<StmtID> {

    assert!(node.kind() == "conditional");
    check_invalid_node(node)?;

    let condition_field = node.child_by_field_name("condition").expect("missing obligatory field");
    let condition = parse_expr(&condition_field, state)?;

    let mut body = Vec::new();
    for_every_child_with_field_name(node, "then_body", |child_node| {
        
        let stmt_id = parse_stmt(&child_node, state)?;
        body.push(stmt_id);
        return Ok(());
    })?;

    let else_body = if node.child_by_field_name("has_else").is_some() {

        let mut else_body = Vec::new();
        for_every_child_with_field_name(node, "else_body", |child_node| {
            
            let stmt_id = parse_stmt(&child_node, state)?;
            else_body.push(stmt_id);
            return Ok(());
        })?;

        Some(else_body)
    }
    else {
        None
    };

    let conditional = Statement::If{cond_expr: condition, then_block: body, else_body: else_body};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_statement_node(conditional, lexical_info);
    return Ok(id);
}

fn parse_var_decl_stmt(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<StmtID> {

    assert_eq!(node.kind(), "variable_declaration");
    check_invalid_node(node)?;

    let is_mut = node.child_by_field_name("is_mut").is_some();

    let name_field = node.child_by_field_name("name").expect("missing obligatory field");
    let name = parse_identifier(&name_field, state)?;

    let type_field = node.child_by_field_name("type").expect("missing obligatory field");
    let type_expr = parse_type_expr(&type_field, state)?;

    let initializer = if let Some(initializer_field) = node.child_by_field_name("initializer") {
        Some(parse_expr(&initializer_field, state)?)
    } else {
        None
    };

    let stmt = Statement::VarDecl{is_mut, name, type_expr, init_expr: initializer};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_statement_node(stmt, lexical_info);
    return Ok(id);
}

fn parse_expr_stmt(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<StmtID> {

    assert_eq!(node.kind(), "expression_stmt");
    check_invalid_node(node)?;

    let expr_field = node.child_by_field_name("expression").expect("missing obligatory field");
    let expr = parse_expr(&expr_field, state)?;

    let stmt = Statement::Expression{expr};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_statement_node(stmt, lexical_info);
    return Ok(id);
}

fn parse_stmt(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<StmtID> {

    let node_type = node.kind();

    match node_type {
        "break_stmt" => {
            return parse_break_stmt(node, state);
        },
        "continue_stmt" => {
            return parse_continue_stmt(node, state);
        },
        "return_stmt" => {
            return parse_return_stmt(node, state);
        },
        "for_each_loop" => {
            return parse_for_each_stmt(node, state);
        },
        "while_loop" => {
            return parse_while_stmt(node, state);
        },
        "conditional" => {
            return parse_conditional_stmt(node, state);
        },
        "variable_declaration" => {
            return parse_var_decl_stmt(node, state);
        },
        "expression_stmt" => {
            return parse_expr_stmt(node, state);
        },
        _ => {
            panic!("Unknown or unexpected node type: {}", node_type);
        }
    }
}

fn parse_function_param(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<FunctionParamID> {

    assert_eq!(node.kind(), "function_param");
    check_invalid_node(node)?;

    let is_mut = node.child_by_field_name("is_mut").is_some();

    let name_field = node.child_by_field_name("name").expect("missing obligatory field");
    let name = parse_identifier(&name_field, state)?;

    let type_field = node.child_by_field_name("type").expect("missing obligatory field");
    let type_expr = parse_type_expr(&type_field, state)?;

    let function_param = FunctionParam{is_mut, name, type_expr};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_function_param_node(function_param, lexical_info);
    return Ok(id);
}

fn parse_function(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<ScopeDefID> {

    assert_eq!(node.kind(), "function");
    check_invalid_node(node)?;

    // parse name
    let name_field = node.child_by_field_name("name").expect("missing obligatory field");
    let name = parse_identifier(&name_field, state)?;


    // parse parameters
    let mut params = Vec::new();
    for_every_child_with_field_name(node, "param", |child_node| {
        
        let param_id = parse_function_param(&child_node, state)?;
        params.push(param_id);
        return Ok(());
    })?;


    // parse return type
    let return_type_field = node.child_by_field_name("return_type").expect("missing obligatory field");
    let return_type = parse_type_expr(&return_type_field, state)?;


    // parse body
    let mut body = Vec::new();
    for_every_child_with_field_name(node, "body", |child_node| {
        
        let stmt_id = parse_stmt(&child_node, state)?;
        body.push(stmt_id);
        return Ok(());
    })?;

    let function = ScopeDef::Function{name, params, return_type, body};
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_scope_def_node(function, lexical_info);
    return Ok(id);
}

fn parse_scope_def(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<ScopeDefID> {

    let node_type = node.kind();

    match node_type {
        "function" => {
            return parse_function(node, state);
        },
        _ => {
            panic!("Unknown or unexpected node type: {}", node_type);
        }
    }
}

fn parse_program(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<()> {

    assert_eq!(node.kind(), "program");
    check_invalid_node(node)?;

    let mut declarations = Vec::new();

    for child in 0..(node.child_count()) {
        let child_node = node.child(child).unwrap();
        let scope_def_id = parse_scope_def(&child_node, state)?;
        declarations.push(scope_def_id);
    }

    state.ast.set_program(Program{global_defs: declarations});
    return Ok(());
}

struct TsoParserState<'a> {
    ast: Ast,
    source: &'a [u8],
}

pub fn parse_tree_sitter_output(tree: &tree_sitter::Tree, source: &str) -> Result<Ast> {

    let mut parser_state = TsoParserState{
        ast: Ast::new(),
        source: source.as_bytes(),
    };

    let root_node = tree.root_node();
    parse_program(&root_node, &mut parser_state)?;
    return Ok(parser_state.ast);
}




/* -------------------------------------------------------------------------- */
/*                              simple test cases                             */
/* -------------------------------------------------------------------------- */

#[cfg(test)]
mod tests {

    use super::*;
    use tree_sitter::Parser;
    use tree_sitter_ila_lang as ila_lang;


    fn is_parsed_successfully(source_code: &str) -> bool {

        let mut parser = Parser::new();
        parser.set_language(&ila_lang::language()).unwrap();

        let tree = parser.parse(source_code, None).unwrap();
        dbg!(tree.root_node().to_sexp());
        return parse_tree_sitter_output(&tree, source_code).is_ok();
    }

    #[test]
    fn test_empty_source() {

        let source_code = "";
        assert!(is_parsed_successfully(source_code));
    }

    #[test]
    fn test_function() {

        let source_code = r#"
            func main() -> int {}
        "#;
        assert!(is_parsed_successfully(source_code));
    }

    #[test]
    fn test_multiple_functions() {

        let source_code = r#"
            func foo() -> int {}
            func bar() -> int {}
        "#;
        assert!(is_parsed_successfully(source_code));
    }

    #[test]
    fn test_function_with_params() {

        let source_code = r#"
            func foo(a: int, b: int) -> int {}
        "#;
        assert!(is_parsed_successfully(source_code));
    }

    #[test]
    fn test_literals() {

        let source_code = r#"
            func foo() -> int {
                10; 0; 1;
                10.0; 0.0; 1.0; 1.01;
                "hello"; "world"; " "; "";
                verdadeiro; falso;
                [1, 2, 3]; [];
                (1);
            }
        "#;
        assert!(is_parsed_successfully(source_code));
    }

    #[test]
    fn test_binary_expr() {

        let source_code = r#"
            func foo() -> int {
                1 + 2; 1 - 2; 1 * 2; 1 / 2;
                1 < 2; 1 > 2; 1 <= 2; 1 >= 2;
                1 == 2; 1 != 2;
                verdadeiro e falso; verdadeiro ou falso;
            }
        "#;
        assert!(is_parsed_successfully(source_code));
    }

    #[test]
    fn test_unary_expr() {

        let source_code = r#"
            func foo() -> int {
                *1; -1; não verdadeiro;
            }
        "#;
        assert!(is_parsed_successfully(source_code));
    }

    #[test]
    fn test_access_expr() {

        let source_code = r#"
            func foo() -> int {
                a.b; a.c.d; a.b.c.d;
            }
        "#;
        assert!(is_parsed_successfully(source_code));
    }

    #[test]
    fn test_call_expr() {

        let source_code = r#"
            func foo() -> int {
                a(); a(b); a(b, c); a(b, c, d);
            }
        "#;
        assert!(is_parsed_successfully(source_code));
    }

    #[test]
    fn test_expr_stmt() {

        let source_code = r#"
            func foo() -> int {
                1; 1 + 2; a.b; a(); a(b);
            }
        "#;
        assert!(is_parsed_successfully(source_code));
    }

    #[test]
    fn test_variable_declaration() {

        let source_code = r#"
            func foo() -> int {
                var a: int; var mut b: int = 2; var c: int = 1;
                var d: int = 1 + 2;
            }
        "#;
        assert!(is_parsed_successfully(source_code));
    }

    #[test]
    fn test_conditional() {

        let source_code = r#"
            func foo() -> int {
                
                se verdadeiro {
                    1;
                }

                se 1 + 2 + 3 > 6 {
                    foobar();
                }

                se falso {
                    2;
                } senão {
                    3;
                }
            }
        "#;
        assert!(is_parsed_successfully(source_code));
    }

    #[test]
    fn test_while_loop() {

        let source_code = r#"
            func foo() -> int {
                
                enquanto verdadeiro {
                    1;
                }

                enquanto foo.size() > 0 {
                    foobar();
                }
            }
        "#;
        assert!(is_parsed_successfully(source_code));
    }

    #[test]
    fn test_for_each_loop() {

        let source_code = r#"
            func foo() -> int {
                
                para cada i em a {
                    1;
                }

                para cada i em foo(a, b, c) {
                    2;
                }

                para cada mut i em a {
                    3;
                }

                para cada ref i em foo(a, b, c) {
                    4;
                }

                para cada mut ref i em a {
                    5;
                }
            }
        "#;
        assert!(is_parsed_successfully(source_code));
    }

    #[test]
    fn test_return_stmt() {

        let source_code = r#"
            func foo() -> int {
                retornar; retornar 1; retornar 1 + 2;
            }
        "#;
        assert!(is_parsed_successfully(source_code));
    }

    #[test]
    fn test_break_stmt() {

        let source_code = r#"
            func foo() -> int {
                parar;
            }
        "#;
        assert!(is_parsed_successfully(source_code));
    }

    #[test]
    fn test_continue_stmt() {

        let source_code = r#"
            func foo() -> int {
                continuar;
            }
        "#;
        assert!(is_parsed_successfully(source_code));
    }

    #[test]
    fn test_raw_type() {

        let source_code = r#"
            func foo() -> int {
                var a: int; var b: float; var c: string;
            }
        "#;
        assert!(is_parsed_successfully(source_code));
    }

    #[test]
    fn test_template_type() {

        let source_code = r#"
            func foo() -> int {
                var a: list<int>; var b: list<list<int>>;
                var m: dict<int, string>;
            }
        "#;
        assert!(is_parsed_successfully(source_code));
    }

    #[test]
    fn test_ref_type() {

        let source_code = r#"
            func foo() -> int {
                var a: ref int; var b: ref mut int;
            }
        "#;
        assert!(is_parsed_successfully(source_code));
    }

    #[test]
    fn test_comp_type() {

        let source_code = r#"
            func foo() -> int {
                var a: comp int; var b: comp mut int;
            }
        "#;
        assert!(is_parsed_successfully(source_code));
    }
}