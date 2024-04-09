
use tree_sitter::Parser;
use tree_sitter_ila_lang as ila_lang;

mod ast;
mod tso_parser;


fn main() {

    let mut parser = Parser::new();
    parser.set_language(&ila_lang::language()).unwrap();

    let source_code = "
    
    func foo(a: abc, b: bcd) -> bar {

        se foo {

        }
        senão {

        }
    }

    ";
    let tree = parser.parse(source_code, None).unwrap();

    println!("{}", tree.root_node().to_string());

    let ast = tso_parser::parse_tree_sitter_output(&tree, source_code).unwrap();
}
