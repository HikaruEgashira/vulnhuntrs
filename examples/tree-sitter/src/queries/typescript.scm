; TypeScript specific definitions
(interface_declaration
  name: (type_identifier) @interface.name)

(type_alias_declaration
  name: (type_identifier) @type.name)

(function_declaration
  name: (identifier) @function.name)

(class_declaration
  name: (type_identifier) @class.name
  body: (class_body) @class.body)

(variable_declarator
  name: (identifier) @variable.name)
