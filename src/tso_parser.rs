
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

fn parse_expr(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<ExprID> {

    let id = parse_identifier(node, state)?;
    let expr = Expression::Identifier { node_id: id };
    let lexical_info = parse_node_lexical_info(node);
    let id = state.ast.add_expression_node(expr, lexical_info);
    return Ok(id);
}

fn parse_break_stmt(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<StmtID> {

    todo!();
}

fn parse_continue_stmt(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<StmtID> {

    todo!();
}

fn parse_return_stmt(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<StmtID> {

    todo!();
}

fn parse_for_each_stmt(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<StmtID> {

    todo!();
}

fn parse_while_stmt(node: &tree_sitter::Node, state: &mut TsoParserState) -> Result<StmtID> {

    todo!();
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
