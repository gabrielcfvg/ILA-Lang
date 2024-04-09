# ILA-Lang

A linguagem ILA-Lang, ou apenas linguagem ILA, é uma linguagem de programação
educacional voltada para programadores iniciantes falantes da língua portuguesa.
A linguagem tem como objetivo tornar o aprendizado de programação mais fácil e
acessível para pessoas que não possuem conhecimento prévio na área, através da
remoção de barreiras linguísticas e da simplificação de conceitos complexos.

A ILA-Lang faz parte do projeto ILA, que tem como proposta a criação de um
ambiente de programação voltado para programadores iniciantes, falantes da
língua portuguesa, incluindo também aqueles que são portadores de deficiências
visuais.

Este documento é uma especificação técnica da linguagem ILA, que provê
informações detalhadas sobre a sintaxe e semântica da linguagem, afim de
possibilitar a implementação de um compilador ou interpretador para a mesma.


## Visão Geral

### Paradigma

ILA-Lang é uma linguagem de programação imperativa, com suporte ao paradigma de
programação estruturada.
\
Programas escritos em ILA-Lang são compostos por subprogramas, que por sua vez
são compostos por uma sequência de instruções, e podem ser de 2 tipos: função ou
procedimento.

##### Instruções:

Todo subprograma é composto por uma sequência de instruções, que são executadas
em ordem. Algumas instruções podem conter sequências de instruções dentro de si,
permitindo que estas sejam executadas de forma condicional, repetitiva, etc.

##### Funções:

Funções são subprogramas que não produzem efeitos colaterais, ou seja, caso a
mesma função seja chamada com os mesmos argumentos, o valor retornado será
sempre o mesmo. Funções obrigatoriamente devem possuir parâmetros e um valor de
retorno.

##### Procedimentos:

Procedimentos são subprogramas que podem produzir efeitos colaterais, ou seja,
podem alterar o estado do programa. Tanto parâmetros quanto valores de retorno
são opcionais em procedimentos, visto que o resultado da computação que eles
realizam pode ser acessado através de efeitos colaterais.


### Escopo e visibilidade da definição de símbolos

ILA-Lang é uma linguagem de 2 estágios("two pass" na literatura), isso significa
símbolos definidos podem ser referenciados em posições no código anteriores à
aquela em que foram definidos. Isso vale pra qualquer tipo de definição,
incluindo a declaração de variáveis.

No caso especial da declaração de variáveis, a variável é acessível em todo o
escopo em que foi declarada, porém, caso ela seja inicializada no *statement* de
declaração, ela só será inicializada com aquele valor quando o *statement* for
"executado".

TODO: caso a variável seja inicializada no *statement* de declaração, prevenir
que algum valor seja atribuído em uma posição anterior à declaração no código.

### Modelo de memória:

auto: liberação automática do objeto. no caso de variáveis locais, a liberação é
realizada quando o escopo em que a variável foi declarada é finalizado. no caso
de membros de uma estrutura, a liberação é realizada quando a estrutura é
liberada.

comp: contagem de referências. a liberação do objeto é realizada quando não há
mais mais referências para o objeto. comp é um 'hard pointer', ou seja, cada
objeto 'comp' apontando para o mesmo objeto, é contabilizado.

ref: referência tradicional, não realiza a liberação do objeto. 'ref' é um 'soft
pointer', ou seja, não é responsável pela liberação do objeto nem é contabilizado
para contagem de referências.

### Sistema de tipos

## Gramática

```

programa = (função)*

função = "func" identificador "(" função_params ")" "->" tipo "{" stmt* "}"
função_params = ( "mut"? identificador ":" tipo ("," "mut"? identificador ":" tipo )* )?

stmt = ( expressão | decl_var | condicional | loop_enquanto | loop_para_cada |
       retornar | continuar | parar ) ";"

decl_var = "var" "mut"? identificador ":" tipo ( "=" expressão )?

condicional = "se" expressão "{" stmt* "}" ( "senão" "{" stmt* "}" )?

loop_enquanto = "enquanto" expressão "{" stmt* "}"

loop_para_cada = "para" "cada" ("mut")? ("ref")? identificador "em" expressão "{" stmt* "}"

retornar = "retornar" expressão

continuar = "continuar"

parar = "parar"


expressão = atrib_expr
atrib_expr = e_expr ( "=" atrib_expr )?
e_expr = ou_expr ( "e" e_expr )?
ou_expr = igual_expr ( "ou" ou_expr )?
igual_expr = rel_expr ( ( "==" | "!=" ) igual_expr )?
rel_expr = termo_expr ( ( "<" | "<=" | ">" | ">=" ) rel_expr )?
termo_expr = fator_expr ( ( "+" | "-" ) termo_expr )?
fator_expr = não_expr ( ( "*" | "/" ) fator_expr )?
não_expr = ( "não" )? neg_expr
neg_expr = ( "-" )? deref_expr
deref_expr = "*" acesso_expr
acesso_expr = valor_expr ( ("." identificador) | "(" (expressão ("," expressão)* )? ")" )*
valor_expr = identificador | literal | "(" expressão ")"
literal = inteiro | decimal | string | booleano | lista_literal
lista_literal = "[" (expressão ("," expressão)*)? "]"

```