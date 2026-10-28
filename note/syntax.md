psudo syntax (compressed):

Literals: num (f32), "str", true/false, nil
Vars: let x = expr
      x = expr
Print: print expr

Blocks: indentation-based (Python-style)
If: if (cond)
        stmt
      else
        stmt
While: while (cond)
         stmt
For: for let i = 0
       i < 3
       i = i + 1
       stmt

Ops (prec high->low): - ! (unary,r) * / (l) + - (l) < > <= >= (l) == != (l) = (assign,r)
Str concat: "a" + "b" -> "ab"  (also "str" + num -> "strnum")

Builtins: readFile(path), writeFile(path,str), listDir(path), exists(path), isFile(path), isDir(path), mkdir(path)
          len(x), push(arr,x), pop(arr), slice(arr,start,end?), keys(obj), values(obj), has(obj,key)
          str(x), num(x), bool(x), clock()
Sandbox: all fs ops confined to ./sandbox/

Classes: class Name
           fn method() ...
         class Derived < Base
           fn method() ...
             super.method()
           ...

Comments: // line, /* block */
End: newline (semicolon optional)
EOF
