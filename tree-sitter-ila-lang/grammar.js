module.exports = grammar({
    name: 'ila_lang',

    word: $ => $.identifier,

    rules: {

        program: $ => repeat($.function),
        
        function: $ => seq(
            'func',
            field('name', $.identifier),
            '(',
            optional(seq(field('param', $.function_param), repeat(seq(',', field('param', $.function_param))))),
            ')',
            '->',
            field('return_type', $._type),
            '{',
            field('body', repeat($._statement)),
            '}',
        ),

        function_param: $ => seq(
            field('is_mut', optional('mut')),
            field('name', $.identifier),
            ':',
            field('type', $._type),
        ),

        _statement: $ => choice(
            $.expression_stmt,
            $.variable_declaration,
            $.conditional,
            $.while_loop,
            $.for_each_loop,
            $.return_stmt,
            $.continue_stmt,
            $.break_stmt,
        ),

        break_stmt: $ => seq('parar', ';'),
        continue_stmt: $ => seq('continuar', ';'),
        return_stmt: $ => seq('retornar', optional(field('return_expr', $._expression)), ';'),

        conditional: $ => seq(
            'se',
            field('condition', $._expression),
            '{',
            field('body', repeat($._statement)),
            '}',
            optional(seq(
                field('has_else', 'senão'),
                '{',
                field('else_body', repeat($._statement)),
                '}'
            )),
        ),

        while_loop: $ => seq(
            'enquanto',
            field('condition', $._expression),
            '{',
            field('body', repeat($._statement)),
            '}',
        ),

        for_each_loop: $ => seq(
            "para", "cada",
            field('item', $.for_item_decl),
            "em",
            field('iterator', $._expression),
            '{',
            field('body', repeat($._statement)),
            '}',
        ),

        for_item_decl: $ => seq(
            field('is_mut', optional('mut')),
            field('is_ref', optional('ref')),
            field('name', $.identifier),
        ),

        variable_declaration: $ => seq(
            'var',
            field('is_mut', optional('mut')),
            field('name', $.identifier),
            ':',
            field('type', $._type),
            optional(seq(
                '=',
                field('initializer', $._expression),
            )),
            ';',
        ),

        expression_stmt: $ => seq(field('expression', $._expression), ';'),


        /* -------------------------------------------------------------------------- */
        /*                                 expressions                                */
        /* -------------------------------------------------------------------------- */

        _expression: $ => choice(
            $.binary_expr,
            $.unary_expr,
            $.access_expr,
            $.call_expr,
            $._value_expr,
        ),

        binary_expr: $ => choice(
            prec.left(1095, seq(field('lhs', $._expression), field('oprt', '='), field('rhs', $._expression))),
            prec.left(1096, seq(field('lhs', $._expression), field('oprt', 'e'), field('rhs', $._expression))),
            prec.left(1096, seq(field('lhs', $._expression), field('oprt', 'ou'), field('rhs', $._expression))),
            prec.left(1097, seq(field('lhs', $._expression), field('oprt', '=='), field('rhs', $._expression))),
            prec.left(1097, seq(field('lhs', $._expression), field('oprt', '!='), field('rhs', $._expression))),
            prec.left(1098, seq(field('lhs', $._expression), field('oprt', '<'), field('rhs', $._expression))),
            prec.left(1098, seq(field('lhs', $._expression), field('oprt', '>'), field('rhs', $._expression))),
            prec.left(1098, seq(field('lhs', $._expression), field('oprt', '<='), field('rhs', $._expression))),
            prec.left(1098, seq(field('lhs', $._expression), field('oprt', '>='), field('rhs', $._expression))),
            prec.left(1099, seq(field('lhs', $._expression), field('oprt', '+'), field('rhs', $._expression))),
            prec.left(1099, seq(field('lhs', $._expression), field('oprt', '-'), field('rhs', $._expression))),
            prec.left(1100, seq(field('lhs', $._expression), field('oprt', '*'), field('rhs', $._expression))),
            prec.left(1100, seq(field('lhs', $._expression), field('oprt', '/'), field('rhs', $._expression))),
        ),

        unary_expr: $ => prec(1102, choice(
            seq(field('oprt', '*'), field('rhs', $._expression)),
            seq(field('oprt', 'não'), field('rhs', $._expression)),
            seq(field('oprt', '-'), field('rhs', $._expression)),
        )),

        access_expr: $ => prec.right(1201, seq(
            field('object', $._expression),
            '.',
            field('item', $.identifier),
        )),

        call_expr: $ => prec(1201, seq(
            field('function', $._expression),
            '(',
            field('arg', optional(seq($._expression, repeat(seq(',', $._expression))))),
            ')',
        )),

        _value_expr: $ => prec(1301, choice(
            $.identifier,
            $._literal,
            $.parem_expr,
        )),

        parem_expr: $ => seq('(', $._expression, ')'),

        _literal: $ => choice(
            $.decimal,
            $.integer,
            $.string,
            $.boolean,
            $.list,
        ),

        integer: $ => /-?[0-9]+/,
        decimal: $ => prec(2000, seq(field("integer", $.integer), '.', field("fraction", /[0-9]+/))),
        string: $ => seq('"', /[^"]*/, '"'),
        boolean: $ => choice('verdadeiro', 'falso'),
        list: $ => seq('[', optional(seq($._expression, repeat(seq(',', $._expression)))), ']'),


        _type: $ => choice(
            $.template_type,
            $.raw_type,
            $.ref_type,
            $.comp_type,
        ),

        template_type: $ => seq(
            field('name', $.identifier),
            '<',
            field('arg', optional(seq($._type, repeat(seq(',', $._type))))),
            '>',
        ),
            
        raw_type: $ => field('name', $.identifier),
        ref_type: $ => seq(field('is_mut', optional('mut')), 'ref', field('type', $._type)),
        comp_type: $ => seq(field('is_mut', optional('mut')), 'comp', field('type', $._type)),


        identifier: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,
    },

    extras: $ => [
        /\s/,
        /#(!\n)*\n/
    ],
});