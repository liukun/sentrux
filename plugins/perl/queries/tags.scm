; Perl tags.scm — verified against actual AST

; functions: subroutine_declaration_statement → bareword field:name
(subroutine_declaration_statement
  name: (bareword) @name) @definition.function

; packages (as classes)
(package_statement
  name: (package) @name) @definition.class

; imports: use_statement → package field:module
(use_statement
  module: (package) @import.module) @import

; require
(require_expression) @import

; ── Calls ──
; Subroutine call: func(args) — function field holds function node (bareword alias)
(function_call_expression
  function: (function) @name) @reference.call

; Method call: $obj->method(args) — method field holds method node
(method_call_expression
  method: (method) @name) @reference.call
