; Official tree-sitter-java tags.scm (v0.23.5)

(class_declaration
  name: (identifier) @name) @definition.class

(method_declaration
  name: (identifier) @name) @definition.method

(method_invocation
  name: (identifier) @name
  arguments: (argument_list) @reference.call)

(interface_declaration
  name: (identifier) @name) @definition.interface

(type_list
  (type_identifier) @name) @reference.implementation

(object_creation_expression
  type: (type_identifier) @name) @reference.class

(superclass (type_identifier) @name) @reference.class

; Enum declarations — pervasive in Java codebases
(enum_declaration
  name: (identifier) @name) @definition.class

; Record declarations (Java 14+)
(record_declaration
  name: (identifier) @name) @definition.class

; ---- Import appendix + custom additions ----

(import_declaration) @import

; Constructors
(constructor_declaration
  name: (identifier) @func.name) @func.def

; Direct function calls: staticMethod() or imported method()
(expression_statement
  (method_invocation
    name: (identifier) @call.name)) @call

; Type references — captures type usage in field declarations, parameters, returns
(type_identifier) @reference.type
