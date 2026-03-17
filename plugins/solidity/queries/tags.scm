; Solidity tags.scm — verified against actual AST

; functions
(function_definition
  name: (identifier) @name) @definition.function

; contracts
(contract_declaration
  name: (identifier) @name) @definition.class

; ── Calls ──
; Function call: func(args) — expression wraps identifier
(call_expression
  (expression
    (identifier) @call.name)) @call

; Member function call: obj.func(args) — expression wraps member_expression
(call_expression
  (expression
    (member_expression
      property: (identifier) @call.name))) @call

; ── Type references ──
; Type usage: user_defined_type wraps identifier(s)
(user_defined_type
  (identifier) @name) @reference.type

; imports: import_directive → string field:source
(import_directive
  source: (string) @import.module) @import
