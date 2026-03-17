; F# tags.scm — verified against actual AST

; functions: function_or_value_defn → function_declaration_left → identifier
(function_or_value_defn
  (function_declaration_left
    (identifier) @name)) @definition.function

; types: type_definition → record_type_defn → type_name → identifier
(type_definition
  (record_type_defn
    (type_name
      (identifier) @name))) @definition.class

; modules: module_defn → identifier
(module_defn
  (identifier) @name) @definition.module

; imports: import_decl → long_identifier
(import_decl
  (long_identifier) @import.module) @import

; ── Calls ──
; Function application: funcName arg — first child is long_identifier_or_op
(application_expression
  (long_identifier_or_op
    (identifier) @name)) @reference.call

; ── Type references ──
; Type annotations and type usage
(type_name
  type_name: (identifier) @name) @reference.type
