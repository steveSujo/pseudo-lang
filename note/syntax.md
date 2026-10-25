psudo syntax (compressed):

Literals: num (f32), "str", true/false, nil
Vars: let x = expr; x = expr;
Print: print expr;
Blocks: { stmt; stmt; }
If: if (cond) { ... } else { ... }
While: while (cond) { ... }
For: for (init; cond; inc) { ... }

Ops (prec high→low): - ! (unary,r) * / (l) + - (l) < > <= >= (l) == != (l) = (assign,r)
Str concat: "a" + "b" → "ab"

Builtins: read_file(path), write_file(path,str), list_dir(path), exists(path), is_file(path), is_dir(path), mkdir(path)
Sandbox: all fs ops confined to ./sandbox/

Comments: // line, /* block */
End: ; required
EOF
