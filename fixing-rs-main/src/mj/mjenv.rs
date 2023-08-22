use super::ast::{parse_ast, MJAstCls};
use fixing_rs_base::{
    containers::{Map, Set},
    grammar::OwnedToken,
    utils::{Pointer, RefArena, StringRef},
};
use getset::{CopyGetters, Getters};
use lalrpop_util::lexer::Token;
use std::{
    cell::RefCell,
    fmt::{Debug, Display},
};

pub struct MJArena<'a> {
    cls: RefArena<MJCls<'a>>,
    field: RefArena<MJField<'a>>,
    constructor: RefArena<MJConstructor<'a>>,
    method: RefArena<MJMethod<'a>>,
    names: RefArena<String>,
}

impl<'a> MJArena<'a> {
    pub fn new() -> Self {
        Self {
            cls: RefArena::new(),
            field: RefArena::new(),
            constructor: RefArena::new(),
            method: RefArena::new(),
            names: RefArena::new(),
        }
    }
}

pub type MJClsRef<'a> = Pointer<'a, MJCls<'a>>;
pub type MJFieldRef<'a> = Pointer<'a, MJField<'a>>;
pub type MJConstructorRef<'a> = Pointer<'a, MJConstructor<'a>>;
pub type MJMethodRef<'a> = Pointer<'a, MJMethod<'a>>;

#[derive(Getters)]
pub struct MJEnv<'a> {
    arena: &'a MJArena<'a>,
    #[get = "pub"]
    names: Map<&'a str, StringRef<'a>>,
    classes: Map<&'a str, MJClsRef<'a>>,
    default_classes: MJDefaultClasses<'a>,
}

const OBJECT_CLASS_NAME: &str = "Object";
const VOID_CLASS_NAME: &str = "void";
const NULL_CLASS_NAME: &str = "null";

#[derive(Debug)]
pub enum MJParseError {
    SyntaxError(lalrpop_util::ParseError<usize, OwnedToken, &'static str>),
    DuplicateClassName(String),
    SuperClassNotFound(String, String),
    FieldTypeNotFound(String, String, String),
    DuplicateField(String, String),
    DuplicateMethod(String, String),
    ParamTypeNotFound(String, String, String),
    InvalidConstructor(String, String),
    ReturnTypeNotFound(String, String, String),
    LoopInh(String),
}

impl Display for MJParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl std::error::Error for MJParseError {}

impl<'input> From<lalrpop_util::ParseError<usize, Token<'input>, &'static str>> for MJParseError {
    fn from(input: lalrpop_util::ParseError<usize, Token<'input>, &'static str>) -> Self {
        Self::SyntaxError(input.map_token(|x| x.into()))
    }
}

pub struct MJDefaultClasses<'a> {
    object: MJClsRef<'a>,
    void: MJClsRef<'a>,
    null: MJClsRef<'a>,
}

impl<'a> MJEnv<'a> {
    pub fn new(arena: &'a MJArena<'a>, ast: Vec<MJAstCls<'_>>) -> Result<Self, MJParseError> {
        let mut names = Map::new();
        let default_classes = Self::allocate_default(arena, &mut names);
        let mut result = Self {
            arena,
            names,
            classes: Map::new(),
            default_classes,
        };
        result.add_default_object();
        result.add_classes(&ast)?;
        result.set_class(&ast)?;
        result.propgrate()?;
        Ok(result)
    }

    fn allocate_cls(
        arena: &'a MJArena<'a>,
        name: &str,
        has_con: bool,
        names: &mut Map<&'a str, StringRef<'a>>,
    ) -> MJClsRef<'a> {
        let name = name.to_string();
        let name = arena.names.alloc(name);
        let result = arena.cls.alloc(MJCls {
            name,
            content: RefCell::new(MJClsContent {
                inh: None,
                fields: Map::new(),
                constructor: None,
                methods: Map::new(),
                supers: Set::new(),
            }),
        });

        if has_con {
            names.insert(name.ptr().as_str(), name);
            result.content.borrow_mut().constructor =
                Some(arena.constructor.alloc(MJConstructor {
                    cls: result,
                    params: Vec::new(),
                }));
        }

        result
    }

    fn allocate_default(
        arena: &'a MJArena<'a>,
        names: &mut Map<&'a str, StringRef<'a>>,
    ) -> MJDefaultClasses<'a> {
        let object = Self::allocate_cls(arena, OBJECT_CLASS_NAME, true, names);
        let void = Self::allocate_cls(arena, VOID_CLASS_NAME, false, names);
        let null = Self::allocate_cls(arena, NULL_CLASS_NAME, false, names);
        MJDefaultClasses { object, void, null }
    }

    pub fn get_name(&mut self, name: &str) -> StringRef<'a> {
        match self.names.get(name) {
            Some(name) => *name,
            None => {
                let name = self.arena.names.alloc(name.to_string());
                self.names.insert(name.ptr(), name);
                name
            }
        }
    }

    pub fn iter_names(&self) -> impl Iterator<Item = &StringRef<'a>> {
        self.names.values()
    }

    fn add_default_object(&mut self) {
        self.classes
            .insert(OBJECT_CLASS_NAME, self.default_classes.object);
    }

    fn add_classes(&mut self, ast: &Vec<MJAstCls<'_>>) -> Result<(), MJParseError> {
        for ast_cls in ast.iter() {
            if self.classes.contains_key(ast_cls.name) {
                return Err(MJParseError::DuplicateClassName(ast_cls.name.to_string()));
            }
            let name = self.get_name(ast_cls.name);
            let cls = self.arena.cls.alloc(MJCls {
                name,
                content: Self::new_content(),
            });
            self.classes.insert(name.ptr(), cls);
        }

        Ok(())
    }

    fn new_content() -> RefCell<MJClsContent<'a>> {
        RefCell::new(MJClsContent {
            inh: None,
            fields: Map::new(),
            constructor: None,
            methods: Map::new(),
            supers: Set::new(),
        })
    }

    fn set_class(&mut self, ast: &Vec<MJAstCls<'_>>) -> Result<(), MJParseError> {
        for ast_cls in ast.iter() {
            let cls = *self.classes.get(ast_cls.name).unwrap();
            let super_cls = self.classes.get(ast_cls.inh);
            match super_cls {
                Some(super_cls) => {
                    cls.content.borrow_mut().inh = Some(*super_cls);
                }
                None => {
                    return Err(MJParseError::SuperClassNotFound(
                        ast_cls.name.to_string(),
                        ast_cls.inh.to_string(),
                    ));
                }
            }

            for ast_field in ast_cls.fields.iter() {
                let ty = self.classes.get(ast_field.ty).copied();
                if cls.content.borrow().fields.contains_key(ast_field.name) {
                    return Err(MJParseError::DuplicateField(
                        ast_cls.name.to_string(),
                        ast_field.name.to_string(),
                    ));
                }
                let name = self.get_name(ast_field.name);
                match ty {
                    Some(ty) => {
                        let field = MJField { cls, name, ty };
                        let field = self.arena.field.alloc(field);
                        cls.content.borrow_mut().fields.insert(name.ptr(), field);
                    }
                    None => {
                        return Err(MJParseError::FieldTypeNotFound(
                            ast_cls.name.to_string(),
                            ast_field.name.to_string(),
                            ast_field.ty.to_string(),
                        ));
                    }
                }
            }

            {
                if ast_cls.constructor.name != ast_cls.name {
                    return Err(MJParseError::InvalidConstructor(
                        ast_cls.name.to_string(),
                        ast_cls.constructor.name.to_string(),
                    ));
                }
                let params: Result<Vec<_>, _> = ast_cls
                    .constructor
                    .params
                    .iter()
                    .map(|x| {
                        let ty = self.classes.get(x).copied();
                        match ty {
                            Some(ty) => Ok(ty),
                            None => Err(MJParseError::ParamTypeNotFound(
                                ast_cls.name.to_string(),
                                "Constructor".to_string(),
                                x.to_string(),
                            )),
                        }
                    })
                    .collect();
                let params = params?;
                let constructor = MJConstructor { cls, params };
                let constructor = self.arena.constructor.alloc(constructor);
                cls.content.borrow_mut().constructor = Some(constructor);
            }

            for ast_method in ast_cls.methods.iter() {
                if cls.content.borrow().methods.contains_key(ast_method.name) {
                    return Err(MJParseError::DuplicateMethod(
                        ast_cls.name.to_string(),
                        ast_method.name.to_string(),
                    ));
                }
                let params: Result<Vec<_>, _> = ast_method
                    .params
                    .iter()
                    .map(|x| {
                        let ty = self.classes.get(x).copied();
                        match ty {
                            Some(ty) => Ok(ty),
                            None => Err(MJParseError::ParamTypeNotFound(
                                ast_cls.name.to_string(),
                                ast_method.name.to_string(),
                                x.to_string(),
                            )),
                        }
                    })
                    .collect();
                let params = params?;
                let ret_ty = match ast_method.ret_ty {
                    Some(ret_ty) => {
                        let ty = self.classes.get(ret_ty).copied();
                        match ty {
                            Some(ty) => Some(ty),
                            None => {
                                return Err(MJParseError::ReturnTypeNotFound(
                                    ast_cls.name.to_string(),
                                    ast_method.name.to_string(),
                                    ret_ty.to_string(),
                                ));
                            }
                        }
                    }
                    None => None,
                };
                let name = self.get_name(ast_method.name);
                let method = MJMethod {
                    cls,
                    name,
                    params,
                    ret_ty,
                };
                let method = self.arena.method.alloc(method);
                cls.content.borrow_mut().methods.insert(name.ptr(), method);
            }
        }

        Ok(())
    }

    fn propgrate(&mut self) -> Result<(), MJParseError> {
        for cls in self.classes.values() {
            let cls = *cls;
            let mut current = cls;
            loop {
                let inh = current.content.borrow().inh;
                let mut content = cls.content.borrow_mut();
                if let Some(inh) = inh {
                    if cls == inh {
                        return Err(MJParseError::LoopInh(cls.name.to_string()));
                    }
                    content.supers.insert(inh);
                    let inh_content = inh.content.borrow();
                    for (name, field) in inh_content.fields.iter() {
                        if !content.fields.contains_key(name) {
                            content.fields.insert(*name, *field);
                        }
                    }
                    for (name, method) in inh_content.methods.iter() {
                        if !content.methods.contains_key(name) {
                            content.methods.insert(*name, *method);
                        }
                    }
                    current = inh;
                } else {
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn can_right_assign_to_left(&self, left: MJClsRef<'a>, right: MJClsRef<'a>) -> bool {
        (left != self.default_classes.void && right != self.default_classes.void)
            && (left == right
                || right == self.default_classes.null
                || right.content.borrow().supers.contains(&left))
    }

    pub fn get_default_object(&self) -> MJClsRef<'a> {
        self.default_classes.object
    }

    pub fn get_default_void(&self) -> MJClsRef<'a> {
        self.default_classes.void
    }

    pub fn get_default_null(&self) -> MJClsRef<'a> {
        self.default_classes.null
    }

    pub fn get_class(&self, name: &str) -> Option<MJClsRef<'a>> {
        self.classes.get(name).copied()
    }

    pub fn iter_class(&self) -> impl Iterator<Item = &MJClsRef<'a>> {
        self.classes.values()
    }

    pub fn build_from_env(arena: &'a MJArena<'a>, env: &str) -> Result<Self, MJParseError> {
        let ast = parse_ast(env)?;
        Self::new(arena, ast)
    }
}

#[derive(Getters)]
pub struct MJCls<'a> {
    #[getset(get = "pub")]
    name: StringRef<'a>,
    #[getset(get = "pub")]
    content: RefCell<MJClsContent<'a>>,
}

impl Debug for MJCls<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MJCls")
            .field("name", &self.name.ptr())
            .finish()
    }
}

impl<'a> MJCls<'a> {
    pub fn constructor(&self) -> MJConstructorRef<'a> {
        self.content.borrow().constructor.unwrap()
    }
}

#[derive(Debug, Getters, CopyGetters)]
pub struct MJClsContent<'a> {
    #[getset(get_copy = "pub")]
    inh: Option<MJClsRef<'a>>,
    #[getset(get = "pub")]
    fields: Map<&'a str, MJFieldRef<'a>>,
    #[getset(get_copy = "pub")]
    constructor: Option<MJConstructorRef<'a>>,
    #[getset(get = "pub")]
    methods: Map<&'a str, MJMethodRef<'a>>,
    #[getset(get = "pub")]
    supers: Set<MJClsRef<'a>>,
}

#[derive(Debug, Getters)]
pub struct MJField<'a> {
    #[getset(get = "pub")]
    cls: MJClsRef<'a>,
    #[getset(get = "pub")]
    name: StringRef<'a>,
    #[getset(get = "pub")]
    ty: MJClsRef<'a>,
}

#[derive(Debug, Getters)]
pub struct MJConstructor<'a> {
    #[getset(get = "pub")]
    cls: MJClsRef<'a>,
    #[getset(get = "pub")]
    params: Vec<MJClsRef<'a>>,
}

#[derive(Debug, Getters)]
pub struct MJMethod<'a> {
    #[getset(get = "pub")]
    cls: MJClsRef<'a>,
    #[getset(get = "pub")]
    name: StringRef<'a>,
    #[getset(get = "pub")]
    params: Vec<MJClsRef<'a>>,
    #[getset(get = "pub")]
    ret_ty: Option<MJClsRef<'a>>,
}
