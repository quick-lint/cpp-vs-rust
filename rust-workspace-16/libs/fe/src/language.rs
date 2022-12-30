#[derive(Clone, Copy, Eq, PartialEq)]
pub enum StatementKind {
    DoWhileLoop,
    ForLoop, // TODO(strager): c_style_for_loop + for_in_loop + for_of_loop?
    IfStatement,
    WhileLoop,
    WithStatement,
    LabelledStatement,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum EnumKind {
    DeclareConstEnum,
    ConstEnum,
    DeclareEnum,
    Normal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VariableKind {
    ArrowParameter,
    Catch,
    Class,
    Const,
    Enum, // TypeScript only
    Function,
    FunctionParameter,     // Non-arrow parameter
    FunctionTypeParameter, // TypeScript only
    GenericParameter,      // TypeScript only
    Import,
    ImportAlias,             // TypeScript only
    ImportType,              // TypeScript only
    IndexSignatureParameter, // TypeScript only
    Interface,               // TypeScript only
    Let,
    Namespace, // TypeScript only
    TypeAlias, // TypeScript only
    Var,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum VariableInitKind {
    // Examples:
    //   class C {}
    //   (param, defaultParam = null) => {}
    //   let x, y, z;
    //   for (let x of xs) {}
    Normal,

    // Only valid for _const, _let, and _var.
    //
    // Examples:
    //   let x = 42;
    //   const [x] = xs;
    //   for (var x = null in xs) {}
    InitializedWithEquals,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FunctionAttributes {
    Async,
    AsyncGenerator,
    Generator,
    Normal,
}
