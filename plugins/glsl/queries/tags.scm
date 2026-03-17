; GLSL tags.scm

(function_definition
  declarator: (function_declarator
    declarator: (identifier) @name)) @definition.function

(struct_specifier
  name: (type_identifier) @name) @definition.class

; ── Calls ──
; Direct function call: func(args)
(call_expression
  function: (identifier) @call.name) @call

; ── Type references ──
(type_identifier) @reference.type

; ---- Import appendix ----

(preproc_include
  path: (_) @import.module) @import
