; Kotlin tags.scm — functions, classes, imports, calls, type references

; ── Definitions ──

(function_declaration
  (simple_identifier) @name) @definition.function

(class_declaration
  (type_identifier) @name) @definition.class

(object_declaration
  (type_identifier) @name) @definition.class

; ── Imports ──

(import_header
  (identifier) @import.module) @import

; ── Calls ──

; Direct call: foo()
(call_expression
  (simple_identifier) @call.name) @call

; Method call: object.method()
(call_expression
  (navigation_expression
    (navigation_suffix
      (simple_identifier) @call.name))) @call

; Constructor call: ClassName(args)
(constructor_invocation
  (user_type
    (type_identifier) @call.name)) @call

; ── Type references ──

(user_type
  (type_identifier) @reference.type)

(delegation_specifier
  (user_type
    (type_identifier) @reference.type))
