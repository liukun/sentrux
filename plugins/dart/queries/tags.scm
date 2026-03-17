; Dart tags.scm — functions, classes, imports, calls, type references

; ── Definitions ──

(function_signature
  name: (identifier) @name) @definition.function

(class_definition
  name: (identifier) @name) @definition.class

(enum_declaration
  name: (identifier) @name) @definition.class

(mixin_declaration
  name: (identifier) @name) @definition.class

; ── Imports ──

(import_or_export
  (library_import
    (import_specification
      (configurable_uri) @import.module))) @import

; ── Calls ──

; Function call: identifier followed by selector with argument_part
(selector
  (argument_part)) @reference.call

; Constructor: new ClassName(args)
(new_expression
  (type_identifier) @call.name) @call

; ── Type references ──

; Type annotations: Type x, List<Type>, Map<K, V>
(type_identifier) @reference.type
