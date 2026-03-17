; Erlang tags.scm — verified against actual AST

; functions: fun_decl → function_clause → atom field:name
(fun_decl
  clause: (function_clause
    name: (atom) @name)) @definition.function

; module attribute
(module_attribute
  name: (atom) @name) @definition.module

; imports: import_attribute → atom field:module
(import_attribute
  module: (atom) @import.module) @import

; ── Calls ──
; Local function call: func(args) — expr field holds the function atom
(call
  expr: (atom) @name) @reference.call

; Remote function call: module:func(args) — expr holds a remote node
(call
  expr: (remote
    fun: (atom) @name)) @reference.call
