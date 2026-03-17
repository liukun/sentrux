; Julia tags.scm — verified against actual AST

; functions: function_definition → signature → call_expression → identifier
(function_definition
  (signature
    (call_expression
      (identifier) @name))) @definition.function

; macros
(macro_definition) @definition.function

; structs: struct_definition → type_head → identifier
(struct_definition
  (type_head
    (identifier) @name)) @definition.class

; abstract types
(abstract_definition) @definition.class

; modules
(module_definition) @definition.module

; imports
(import_statement
  (identifier) @import.module) @import
(using_statement) @import

; ── Calls ──
; Direct function call: func(args)
(call_expression
  (identifier) @name) @reference.call

; Method/qualified call: Module.func(args)
(call_expression
  (field_expression
    (identifier) @name)) @reference.call

; ── Type references ──
; Type annotations: x::Type
(typed_expression
  (identifier) @name) @reference.type

; Parametric types: Vector{Int}
(parametrized_type_expression
  (identifier) @name) @reference.type
