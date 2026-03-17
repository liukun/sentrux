; V language tags.scm

(function_declaration
  name: (identifier) @name) @definition.function

(struct_declaration
  name: (identifier) @name) @definition.class

(enum_declaration
  name: (identifier) @name) @definition.class

(interface_declaration
  name: (identifier) @name) @definition.interface

; ── Calls ──
; Direct function call: func(args)
(call_expression
  name: (reference_expression
    (identifier) @call.name)) @call

; Method/qualified call: obj.method(args)
(call_expression
  name: (selector_expression
    field: (reference_expression
      (identifier) @call.name))) @call

; ── Type references ──
(type_reference_expression
  (identifier) @name) @reference.type

; ---- Import appendix ----

(import_declaration
  (import_path) @import.module) @import
