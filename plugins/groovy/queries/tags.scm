; Groovy tags.scm — verified against actual AST

; functions: function_definition → identifier field:function
(function_definition
  function: (identifier) @name) @definition.function

(function_declaration
  function: (identifier) @name) @definition.function

; classes: class_definition → identifier field:name
(class_definition
  name: (identifier) @name) @definition.class

; imports: groovy_import → qualified_name field:import
(groovy_import
  import: (qualified_name) @import.module) @import

; ── Calls ──
; Direct function/method call
(function_call
  function: (identifier) @name) @reference.call

; Method call on object: obj.method(args)
(function_call
  function: (dotted_identifier
    (identifier) @name)) @reference.call

; ── Type references ──
; Parameterized type: List<String>, Map<K,V>
(type_with_generics
  (identifier) @name) @reference.type
