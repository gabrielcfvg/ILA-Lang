
use std::collections::HashMap;


#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct NodeID { id: usize }
pub struct ScopeDefID(NodeID);
pub struct StmtID(NodeID);
pub struct ExprID(NodeID);
pub struct TypeExprID(NodeID);
pub struct IdentifierID(NodeID);
pub struct FunctionParamID(NodeID);


pub struct Ast {
    
    node_id_counter: usize,
    node_lexical_info: HashMap<NodeID, LexicalInfo>,
    node_value: HashMap<NodeID, Node>,
    program: Option<Program>,
}

impl Ast {

    pub fn new() -> Self {
        Ast {
            node_id_counter: 0,
            node_lexical_info: HashMap::new(),
            node_value: HashMap::new(),
            program: None,
        }
    }

    fn get_node_id(&mut self) -> NodeID {
        let id = self.node_id_counter;
        self.node_id_counter += 1;
        return NodeID{id};
    }

    /* -------------------------------------------------------------------------- */
    /*                           ast building functions                           */
    /* -------------------------------------------------------------------------- */

    pub fn add_node(&mut self, node: Node, lexical_info: LexicalInfo) -> NodeID {
        let id = self.get_node_id();
        self.node_lexical_info.insert(id, lexical_info);
        self.node_value.insert(id, node);
        return id;
    }

    pub fn add_scope_def_node(&mut self, scope_def: ScopeDef, lexical_info: LexicalInfo) -> ScopeDefID {
        let id = self.add_node(Node::ScopeDef(scope_def), lexical_info);
        return ScopeDefID(id);
    }

    pub fn add_statement_node(&mut self, statement: Statement, lexical_info: LexicalInfo) -> StmtID {
        let id = self.add_node(Node::Statement(statement), lexical_info);
        return StmtID(id);
    }

    pub fn add_expression_node(&mut self, expression: Expression, lexical_info: LexicalInfo) -> ExprID {
        let id = self.add_node(Node::Expression(expression), lexical_info);
        return ExprID(id);
    }

    pub fn add_type_expr_node(&mut self, type_expr: TypeExpr, lexical_info: LexicalInfo) -> TypeExprID {
        let id = self.add_node(Node::TypeExpr(type_expr), lexical_info);
        return TypeExprID(id);
    }

    pub fn add_identifier_node(&mut self, name: String, lexical_info: LexicalInfo) -> IdentifierID {
        let id = self.add_node(Node::Identifier(name), lexical_info);
        return IdentifierID(id);
    }

    pub fn add_function_param_node(&mut self, function_param: FunctionParam, lexical_info: LexicalInfo) -> FunctionParamID {
        let id = self.add_node(Node::FunctionParam(function_param), lexical_info);
        return FunctionParamID(id);
    }


    pub fn set_program(&mut self, program: Program) {
        self.program = Some(program);
    }

    
    /* -------------------------------------------------------------------------- */
    /*                         ast manipulation functions                         */
    /* -------------------------------------------------------------------------- */

    pub fn get_node(&self, id: NodeID) -> &Node {
        return self.node_value.get(&id).expect("NodeID does not point to a node");
    }

    pub fn get_scope_def(&self, id: ScopeDefID) -> &ScopeDef {
        return match self.get_node(id.0) {
            Node::ScopeDef(scope_def) => scope_def,
            _ => panic!("ScopeDefID does not point to a ScopeDef node"),
        }
    }

    pub fn get_statement(&self, id: StmtID) -> &Statement {
        return match self.get_node(id.0) {
            Node::Statement(statement) => statement,
            _ => panic!("StmtID does not point to a Statement node"),
        }
    }

    pub fn get_expression(&self, id: ExprID) -> &Expression {
        return match self.get_node(id.0) {
            Node::Expression(expression) => expression,
            _ => panic!("ExprID does not point to a Expression node"),
        }
    }

    pub fn get_type_expr(&self, id: TypeExprID) -> &TypeExpr {
        return match self.get_node(id.0) {
            Node::TypeExpr(type_expr) => type_expr,
            _ => panic!("TypeExprID does not point to a TypeExpr node"),
        }
    }

    pub fn get_identifier(&self, id: IdentifierID) -> &String {
        return match self.get_node(id.0) {
            Node::Identifier(name) => name,
            _ => panic!("IdentifierID does not point to a Identifier node"),
        }
    }

    pub fn get_function_param(&self, id: FunctionParamID) -> &FunctionParam {
        return match self.get_node(id.0) {
            Node::FunctionParam(param) => param,
            _ => panic!("FunctionParamID does not point to a FunctionParam node"),
        }
    }
}



pub struct LexicalPosition {
    pub line: usize,
    pub column: usize,
}

pub struct LexicalInfo {
    pub start: LexicalPosition,
    pub end: LexicalPosition,
}


pub struct Program {
    pub global_defs: Vec<ScopeDefID>,
}

enum Node {
    ScopeDef(ScopeDef),
    Statement(Statement),
    Expression(Expression),
    TypeExpr(TypeExpr),
    Identifier(String),
    FunctionParam(FunctionParam),
}

pub enum ScopeDef {
    Function{name: IdentifierID, params: Vec<FunctionParamID>, return_type: TypeExprID, body: Vec<StmtID>},
}

pub enum Statement {
    Expression{expr: ExprID},
    VarDecl{is_mut: bool, name: IdentifierID, type_expr: TypeExprID, init_expr: Option<ExprID>},
    If{cond_expr: ExprID, then_block: Vec<StmtID>, else_body: Option<Vec<StmtID>>},
    While{cond_expr: ExprID, body_block: Vec<StmtID>},
    ForEach{is_var_mut: bool, is_var_ref: bool, var_name: NodeID, iter_expr: ExprID, body_block: Vec<StmtID>},
    Return{expr: Option<ExprID>},
    Continue,
    Break,
}

pub enum Expression {
    IntegerLiteral{value: i64}, // TODO: usar tipo sem limite de precisão
    DecimalLiteral{value: f64}, // TODO: usar tipo sem limite de precisão
    StringLiteral{value: String},
    BooleanLiteral{value: bool},
    ListLiteral{values: Vec<ExprID>},
    Identifier{node_id: IdentifierID},
    Call{callee: ExprID, args: Vec<ExprID>},
    Access{object: ExprID, field_name: NodeID},
    BinaryOprt{oprt: BinaryOprt, left: ExprID, right: ExprID},
    UnaryOprt{oprt: UnaryOprt, operand: ExprID},
    Assign{target: ExprID, value: ExprID},
}

pub enum TypeExpr {

    RawType{type_name: IdentifierID},
    TemplateType{type_name: IdentifierID, type_args: Vec<TypeExprID>},
    RefType{is_mut: bool, type_expr: TypeExprID},
    CompType{is_mut: bool, type_expr: TypeExprID},
}

pub enum UnaryOprt {
    Neg,
    Not,
    Deref
}

pub enum BinaryOprt {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
}


pub struct FunctionParam {
    pub is_mut: bool,
    pub name: IdentifierID,
    pub type_expr: TypeExprID,
}
