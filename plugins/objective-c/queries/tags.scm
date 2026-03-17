; Objective-C tags.scm — verified against actual AST

; functions: function_definition → function_declarator → identifier
(function_definition
  declarator: (function_declarator
    declarator: (identifier) @name)) @definition.function

; methods (captured without name extraction — ObjC method names are complex)
(method_declaration) @definition.function

; classes: class_interface → identifier (direct child, no field name)
(class_interface
  (identifier) @name) @definition.class

; protocols
(protocol_declaration
  (identifier) @name) @definition.interface

; ── Calls ──
; C-style function call: func(args)
(call_expression
  function: (identifier) @call.name) @call

; C-style member call: obj->func(args) or obj.func(args)
(call_expression
  function: (field_expression
    field: (field_identifier) @call.name)) @call

; ObjC message send: [object method:arg]
(message_expression
  method: (identifier) @call.name) @call

; ── Type references ──
(type_identifier) @reference.type

; imports: preproc_include with field:path
(preproc_include
  path: (_) @import.module) @import
