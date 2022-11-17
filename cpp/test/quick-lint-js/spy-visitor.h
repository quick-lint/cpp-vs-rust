// Copyright (C) 2020  Matthew "strager" Glazar
// See end of file for extended copyright information.

#ifndef QUICK_LINT_JS_SPY_VISITOR_H
#define QUICK_LINT_JS_SPY_VISITOR_H

#include <iosfwd>
#include <optional>
#include <quick-lint-js/diag-collector.h>
#include <quick-lint-js/fe/language.h>
#include <quick-lint-js/fe/lex.h>
#include <quick-lint-js/gtest.h>
#include <quick-lint-js/port/char8.h>
#include <string_view>
#include <vector>

namespace quick_lint_js {
struct visited_variable_declaration {
  string8 name;
  variable_kind kind;
  variable_init_kind init_kind;

  bool operator==(const visited_variable_declaration &other) const {
    return this->name == other.name && this->kind == other.kind &&
           this->init_kind == other.init_kind;
  }

  bool operator!=(const visited_variable_declaration &other) const {
    return !(*this == other);
  }
};

// An function/method parameter. Not an arrow function parameter.
inline visited_variable_declaration arrow_param_decl(string8_view name) {
  return visited_variable_declaration{string8(name),
                                      variable_kind::_arrow_parameter,
                                      variable_init_kind::normal};
}

inline visited_variable_declaration catch_decl(string8_view name) {
  return visited_variable_declaration{string8(name), variable_kind::_catch,
                                      variable_init_kind::normal};
}

inline visited_variable_declaration class_decl(string8_view name) {
  return visited_variable_declaration{string8(name), variable_kind::_class,
                                      variable_init_kind::normal};
}

// A variable declared with 'const' with an initializer. Example: const x =
// null;
inline visited_variable_declaration const_init_decl(string8_view name) {
  return visited_variable_declaration{
      string8(name), variable_kind::_const,
      variable_init_kind::initialized_with_equals};
}

// A variable declared with 'const' without an initializer.
// Example: for (const x of []) {}
inline visited_variable_declaration const_noinit_decl(string8_view name) {
  return visited_variable_declaration{string8(name), variable_kind::_const,
                                      variable_init_kind::normal};
}

inline visited_variable_declaration enum_decl(string8_view name) {
  return visited_variable_declaration{string8(name), variable_kind::_enum,
                                      variable_init_kind::normal};
}

inline visited_variable_declaration function_decl(string8_view name) {
  return visited_variable_declaration{string8(name), variable_kind::_function,
                                      variable_init_kind::normal};
}

// An function/method parameter. Not an arrow function parameter.
inline visited_variable_declaration func_param_decl(string8_view name) {
  return visited_variable_declaration{string8(name),
                                      variable_kind::_function_parameter,
                                      variable_init_kind::normal};
}

// An function parameter in a TypeScript type.
inline visited_variable_declaration func_type_param_decl(string8_view name) {
  return visited_variable_declaration{string8(name),
                                      variable_kind::_function_type_parameter,
                                      variable_init_kind::normal};
}

// A TypeScript namespace or module alias. Example: import A = B;
inline visited_variable_declaration import_alias_decl(string8_view name) {
  return visited_variable_declaration{
      string8(name), variable_kind::_import_alias, variable_init_kind::normal};
}

inline visited_variable_declaration import_decl(string8_view name) {
  return visited_variable_declaration{string8(name), variable_kind::_import,
                                      variable_init_kind::normal};
}

inline visited_variable_declaration import_type_decl(string8_view name) {
  return visited_variable_declaration{
      string8(name), variable_kind::_import_type, variable_init_kind::normal};
}

// A parameter in a TypeScript index signature.
//
// Example: [key: KeyType]: ValueType  // key is an index signature parameter.
inline visited_variable_declaration index_signature_param_decl(
    string8_view name) {
  return visited_variable_declaration{string8(name),
                                      variable_kind::_index_signature_parameter,
                                      variable_init_kind::normal};
}

inline visited_variable_declaration interface_decl(string8_view name) {
  return visited_variable_declaration{string8(name), variable_kind::_interface,
                                      variable_init_kind::normal};
}

// A variable declared with 'let' with an initializer. Example: let x = null;
inline visited_variable_declaration let_init_decl(string8_view name) {
  return visited_variable_declaration{
      string8(name), variable_kind::_let,
      variable_init_kind::initialized_with_equals};
}

// A variable declared with 'let' without an initializer. Example: let x;
inline visited_variable_declaration let_noinit_decl(string8_view name) {
  return visited_variable_declaration{string8(name), variable_kind::_let,
                                      variable_init_kind::normal};
}

// A TypeScript namespace (declared with the 'namespace' keyword).
inline visited_variable_declaration namespace_decl(string8_view name) {
  return visited_variable_declaration{string8(name), variable_kind::_namespace,
                                      variable_init_kind::normal};
}

// A TypeScript generic function parameter.
inline visited_variable_declaration generic_param_decl(string8_view name) {
  return visited_variable_declaration{string8(name),
                                      variable_kind::_generic_parameter,
                                      variable_init_kind::normal};
}

// A TypeScript type alias. Example: type T = number;
inline visited_variable_declaration type_alias_decl(string8_view name) {
  return visited_variable_declaration{string8(name), variable_kind::_type_alias,
                                      variable_init_kind::normal};
}

// A variable declared with 'var' with an initializer. Example: var x = null;
inline visited_variable_declaration var_init_decl(string8_view name) {
  return visited_variable_declaration{
      string8(name), variable_kind::_var,
      variable_init_kind::initialized_with_equals};
}

// A variable declared with 'var' without an initializer. Example: var x;
inline visited_variable_declaration var_noinit_decl(string8_view name) {
  return visited_variable_declaration{string8(name), variable_kind::_var,
                                      variable_init_kind::normal};
}

// TODO(strager): Rename this.
struct spy_visitor final : public diag_collector {};

void PrintTo(const visited_variable_declaration &, std::ostream *);
}

#endif

// quick-lint-js finds bugs in JavaScript programs.
// Copyright (C) 2020  Matthew "strager" Glazar
//
// This file is part of quick-lint-js.
//
// quick-lint-js is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// quick-lint-js is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with quick-lint-js.  If not, see <https://www.gnu.org/licenses/>.
