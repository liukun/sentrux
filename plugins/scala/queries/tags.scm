; Scala tags.scm — functions, classes, imports, calls, type references

; ── Definitions ──

(function_definition
  name: (identifier) @name) @definition.function

(class_definition
  name: (identifier) @name) @definition.class

(object_definition
  name: (identifier) @name) @definition.class

(trait_definition
  name: (identifier) @name) @definition.class

; ── Imports ──

(import_declaration) @import

; ── Calls ──

(call_expression
  function: (identifier) @call.name) @call

(call_expression
  function: (field_expression
    field: (identifier) @call.name)) @call

; ── Type references ──

(type_identifier) @reference.type
