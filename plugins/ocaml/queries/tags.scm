; OCaml tags.scm — verified against actual AST

; functions: value_definition → let_binding → value_name field:pattern
(value_definition
  (let_binding
    pattern: (value_name) @name)) @definition.function

; types: type_definition → type_binding → type_constructor field:name
(type_definition
  (type_binding
    name: (type_constructor) @name)) @definition.class

; modules: module_definition → module_binding → module_name
(module_definition
  (module_binding
    (module_name) @name)) @definition.module

; imports: open_module → module_path field:module → module_name
(open_module
  module: (module_path
    (module_name) @import.module)) @import

; ── Calls ──
; Function application: func arg
(application_expression
  function: (value_path
    (value_name) @name)) @reference.call

; Qualified call: Module.func
(application_expression
  function: (value_path
    (module_path
      (module_name))
    (value_name) @name)) @reference.call

; Constructor application (type reference)
(constructor_path
  (constructor_name) @name) @reference.type
