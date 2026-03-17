; Official tree-sitter-typescript tags.scm (v0.23.2) + inlined JS base patterns

; ---- TS-specific captures ----

(function_signature
  name: (identifier) @name) @definition.function

(method_signature
  name: (property_identifier) @name) @definition.method

(abstract_method_signature
  name: (property_identifier) @name) @definition.method

(abstract_class_declaration
  name: (type_identifier) @name) @definition.class

(module
  name: (identifier) @name) @definition.module

(interface_declaration
  name: (type_identifier) @name) @definition.interface

(new_expression
  constructor: (identifier) @name) @reference.class

; ---- JS base patterns (TS inherits JS grammar) ----

(
  (comment)* @doc
  .
  (method_definition
    name: (property_identifier) @name) @definition.method
  (#not-eq? @name "constructor")
  (#strip! @doc "^[\\s\\*/]+|^[\\s\\*/]$")
  (#select-adjacent! @doc @definition.method)
)

(
  (comment)* @doc
  .
  [
    (class
      name: (_) @name)
    (class_declaration
      name: (_) @name)
  ] @definition.class
  (#strip! @doc "^[\\s\\*/]+|^[\\s\\*/]$")
  (#select-adjacent! @doc @definition.class)
)

(
  (comment)* @doc
  .
  [
    (function_expression
      name: (identifier) @name)
    (function_declaration
      name: (identifier) @name)
    (generator_function
      name: (identifier) @name)
    (generator_function_declaration
      name: (identifier) @name)
  ] @definition.function
  (#strip! @doc "^[\\s\\*/]+|^[\\s\\*/]$")
  (#select-adjacent! @doc @definition.function)
)

(
  (comment)* @doc
  .
  (lexical_declaration
    (variable_declarator
      name: (identifier) @name
      value: [(arrow_function) (function_expression)]) @definition.function)
  (#strip! @doc "^[\\s\\*/]+|^[\\s\\*/]$")
  (#select-adjacent! @doc @definition.function)
)

(
  (comment)* @doc
  .
  (variable_declaration
    (variable_declarator
      name: (identifier) @name
      value: [(arrow_function) (function_expression)]) @definition.function)
  (#strip! @doc "^[\\s\\*/]+|^[\\s\\*/]$")
  (#select-adjacent! @doc @definition.function)
)

(assignment_expression
  left: [
    (identifier) @name
    (member_expression
      property: (property_identifier) @name)
  ]
  right: [(arrow_function) (function_expression)]
) @definition.function

(pair
  key: (property_identifier) @name
  value: [(arrow_function) (function_expression)]) @definition.function

(
  (call_expression
    function: (identifier) @name) @reference.call
  (#not-match? @name "^(require)$")
)

(call_expression
  function: (member_expression
    property: (property_identifier) @name)
  arguments: (_) @reference.call)

; ---- TS-specific definitions ----

; Type aliases: type Foo = ...
(type_alias_declaration
  name: (type_identifier) @name) @definition.class

; Enum declarations
(enum_declaration
  name: (identifier) @name) @definition.class

; Type references
(type_identifier) @reference.type

; ---- Import appendix (custom) ----

; ES6 import
(import_statement
  source: (string) @import.module) @import

; CommonJS require()
(call_expression
  function: (identifier) @_fn
  arguments: (arguments (string) @import.module)
  (#eq? @_fn "require")) @import

; Dynamic import()
(call_expression
  function: (import)
  arguments: (arguments (string) @import.module)) @import

; Re-exports
(export_statement
  source: (string) @import.module) @import
