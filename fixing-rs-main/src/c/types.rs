use fixing_rs_base::utils::{Pointer, RefArena};
use getset::{CopyGetters, Getters};
use std::{cell::RefCell, collections::HashMap, fmt::Debug};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Prop, ValueEnum)]
pub enum CTypeToken {
    Signed,
    Unsigned,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Const,
    Void,
    LongLong,
    LongDouble,
}

impl CTypeToken {
    pub const fn bit_loc(&self) -> u32 {
        match self {
            CTypeToken::Signed => 0,
            CTypeToken::Unsigned => 1,
            CTypeToken::Char => 2,
            CTypeToken::Short => 3,
            CTypeToken::Int => 4,
            CTypeToken::Long => 5,
            CTypeToken::Float => 6,
            CTypeToken::Double => 7,
            CTypeToken::Const => 8,
            CTypeToken::Void => 9,
            CTypeToken::LongLong => 10,
            CTypeToken::LongDouble => 11,
        }
    }

    pub const fn bit_val(&self) -> u32 {
        1 << self.bit_loc()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Prop)]
pub struct CTypeComposed {
    types: u32,
}

impl CTypeComposed {
    pub fn new() -> Self {
        Self { types: 0 }
    }

    pub fn from_token(token: CTypeToken) -> Self {
        Self {
            types: token.bit_val(),
        }
    }

    pub fn has_token(&self, token: CTypeToken) -> bool {
        self.types & token.bit_val() != 0
    }

    pub fn append_token(&self, token: CTypeToken) -> Option<Self> {
        match token {
            CTypeToken::Signed => {
                if self.has_token(CTypeToken::Signed)
                    || self.has_token(CTypeToken::Unsigned)
                    || self.has_token(CTypeToken::Float)
                    || self.has_token(CTypeToken::Double)
                    || self.has_token(CTypeToken::Void)
                {
                    return None;
                }
            }
            CTypeToken::Unsigned => {
                if self.has_token(CTypeToken::Signed)
                    || self.has_token(CTypeToken::Unsigned)
                    || self.has_token(CTypeToken::Float)
                    || self.has_token(CTypeToken::Double)
                    || self.has_token(CTypeToken::Void)
                {
                    return None;
                }
            }
            CTypeToken::Char => {
                if self.has_token(CTypeToken::Char)
                    || self.has_token(CTypeToken::Int)
                    || self.has_token(CTypeToken::Short)
                    || self.has_token(CTypeToken::Long)
                    || self.has_token(CTypeToken::Float)
                    || self.has_token(CTypeToken::Double)
                    || self.has_token(CTypeToken::Void)
                {
                    return None;
                }
            }
            CTypeToken::Short => {
                if self.has_token(CTypeToken::Char)
                    || self.has_token(CTypeToken::Short)
                    || self.has_token(CTypeToken::Long)
                    || self.has_token(CTypeToken::Float)
                    || self.has_token(CTypeToken::Double)
                    || self.has_token(CTypeToken::Void)
                {
                    return None;
                }
            }
            CTypeToken::Int => {
                if self.has_token(CTypeToken::Char)
                    || self.has_token(CTypeToken::Int)
                    || self.has_token(CTypeToken::Float)
                    || self.has_token(CTypeToken::Double)
                    || self.has_token(CTypeToken::Void)
                {
                    return None;
                }
            }
            CTypeToken::Long => {
                if self.has_token(CTypeToken::Char)
                    || self.has_token(CTypeToken::Short)
                    || self.has_token(CTypeToken::Float)
                    || self.has_token(CTypeToken::Void)
                    || self.has_token(CTypeToken::LongLong)
                    || self.has_token(CTypeToken::LongDouble)
                {
                    return None;
                }
                if self.has_token(CTypeToken::Long) {
                    return Some(Self {
                        types: self.types | CTypeToken::LongLong.bit_val(),
                    });
                }
                if self.has_token(CTypeToken::Double) {
                    return Some(Self {
                        types: self.types
                            | CTypeToken::LongDouble.bit_val()
                            | CTypeToken::Long.bit_val(),
                    });
                }
            }
            CTypeToken::Float => {
                if self.has_token(CTypeToken::Signed)
                    || self.has_token(CTypeToken::Unsigned)
                    || self.has_token(CTypeToken::Char)
                    || self.has_token(CTypeToken::Int)
                    || self.has_token(CTypeToken::Short)
                    || self.has_token(CTypeToken::Long)
                    || self.has_token(CTypeToken::Float)
                    || self.has_token(CTypeToken::Double)
                    || self.has_token(CTypeToken::Void)
                {
                    return None;
                }
            }
            CTypeToken::Double => {
                if self.has_token(CTypeToken::Signed)
                    || self.has_token(CTypeToken::Unsigned)
                    || self.has_token(CTypeToken::Char)
                    || self.has_token(CTypeToken::Int)
                    || self.has_token(CTypeToken::Short)
                    || self.has_token(CTypeToken::Float)
                    || self.has_token(CTypeToken::Double)
                    || self.has_token(CTypeToken::Void)
                {
                    return None;
                }
                if self.has_token(CTypeToken::Long) {
                    return Some(Self {
                        types: self.types
                            | CTypeToken::LongDouble.bit_val()
                            | CTypeToken::Long.bit_val(),
                    });
                }
            }
            CTypeToken::Const => {
                if self.has_token(CTypeToken::Const) {
                    return None;
                }
            }
            CTypeToken::Void => {
                if self.has_token(CTypeToken::Signed)
                    || self.has_token(CTypeToken::Unsigned)
                    || self.has_token(CTypeToken::Char)
                    || self.has_token(CTypeToken::Int)
                    || self.has_token(CTypeToken::Short)
                    || self.has_token(CTypeToken::Long)
                    || self.has_token(CTypeToken::Float)
                    || self.has_token(CTypeToken::Double)
                    || self.has_token(CTypeToken::Void)
                {
                    return None;
                }
            }
            CTypeToken::LongLong | CTypeToken::LongDouble => unreachable!(),
        }

        Some(Self {
            types: self.types | token.bit_val(),
        })
    }
}

pub type CTypeRef<'a> = Pointer<'a, CType<'a>>;

pub struct CTypeArena<'a> {
    types: RefArena<CType<'a>>,
}

impl<'a> CTypeArena<'a> {
    pub fn new() -> Self {
        Self {
            types: RefArena::new(),
        }
    }
}

pub struct CTypeBasicTypes<'a> {
    pub void: CTypeRef<'a>,
    pub int: CTypeRef<'a>,
    pub float: CTypeRef<'a>,
}

#[derive(Getters)]
pub struct CTypePool<'a> {
    arena: &'a CTypeArena<'a>,
    #[get = "pub"]
    basic_types: CTypeBasicTypes<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
pub enum CBasicType {
    Void,
    Int,
    Float,
}

impl CBasicType {
    pub fn name(&self) -> &'static str {
        match self {
            CBasicType::Void => "void",
            CBasicType::Int => "int",
            CBasicType::Float => "float",
        }
    }
}

impl<'a> CTypePool<'a> {
    pub fn new(arena: &'a CTypeArena<'a>) -> Self {
        let basic_types = CTypeBasicTypes {
            void: arena.types.alloc(CType::new_basic(CBasicType::Void)),
            int: arena.types.alloc(CType::new_basic(CBasicType::Int)),
            float: arena.types.alloc(CType::new_basic(CBasicType::Float)),
        };
        Self { arena, basic_types }
    }

    fn alloc_with_content(&self, content: CTypeContent<'a>) -> CTypeRef<'a> {
        self.arena.types.alloc(CType::new(content))
    }

    pub fn derive_const(&self, ty: CTypeRef<'a>) -> Option<CTypeRef<'a>> {
        if let Some(x) = ty.derive.borrow().derive_const {
            return Some(x);
        } else if let CTypeContent::Const(_) = ty.content {
            return None;
        }
        let new_ty = self.alloc_with_content(CTypeContent::Const(ty));
        ty.derive.borrow_mut().derive_const = Some(new_ty);
        Some(new_ty)
    }

    pub fn derive_pointer(&self, ty: CTypeRef<'a>) -> Option<CTypeRef<'a>> {
        if let Some(x) = ty.derive.borrow().derive_pointer {
            return Some(x);
        }
        let new_ty = self.alloc_with_content(CTypeContent::Pointer(ty));
        ty.derive.borrow_mut().derive_pointer = Some(new_ty);
        Some(new_ty)
    }

    pub fn derive_array(&self, ty: CTypeRef<'a>) -> Option<CTypeRef<'a>> {
        if let Some(x) = ty.derive.borrow().derive_array {
            return Some(x);
        } else if !ty.is_complete() {
            return None;
        }
        let new_ty = self.alloc_with_content(CTypeContent::Array(ty));
        ty.derive.borrow_mut().derive_array = Some(new_ty);
        Some(new_ty)
    }

    pub fn derive_imcomplete_array(&self, ty: CTypeRef<'a>) -> Option<CTypeRef<'a>> {
        if let Some(x) = ty.derive.borrow().derive_imcomplete_array {
            return Some(x);
        } else if !ty.is_complete() {
            return None;
        }
        let new_ty = self.alloc_with_content(CTypeContent::IncompleteArray(ty));
        ty.derive.borrow_mut().derive_imcomplete_array = Some(new_ty);
        Some(new_ty)
    }

    pub fn derive_func(
        &self,
        ret: CTypeRef<'a>,
        args: Vec<CTypeRef<'a>>,
        va_args: bool,
    ) -> CTypeRef<'a> {
        if let Some(x) = ret.derive.borrow().derive_func.get(&(va_args, &args[..])) {
            return *x;
        }
        let new_ty =
            self.alloc_with_content(CTypeContent::Func(CFuncContent { ret, args, va_args }));
        let new_ty_args = match &new_ty.ptr().content {
            CTypeContent::Func(CFuncContent { ref args, .. }) => args,
            _ => unreachable!(),
        };
        ret.derive
            .borrow_mut()
            .derive_func
            .insert((va_args, &new_ty_args[..]), new_ty);
        new_ty
    }

    pub fn get_func_content(&self, ty: CTypeRef<'a>) -> Option<CFuncContentRef<'a>> {
        match ty.ptr().content {
            CTypeContent::Func(ref content) => Some(Pointer::from_ptr(content)),
            _ => None,
        }
    }

    pub fn type_u8(&self) -> CTypeRef<'a> {
        self.basic_types.int
    }
    pub fn type_u16(&self) -> CTypeRef<'a> {
        self.basic_types.int
    }
    pub fn type_u32(&self) -> CTypeRef<'a> {
        self.basic_types.int
    }
    pub fn type_u64(&self) -> CTypeRef<'a> {
        self.basic_types.int
    }
    pub fn type_i8(&self) -> CTypeRef<'a> {
        self.basic_types.int
    }
    pub fn type_i16(&self) -> CTypeRef<'a> {
        self.basic_types.int
    }
    pub fn type_i32(&self) -> CTypeRef<'a> {
        self.basic_types.int
    }
    pub fn type_i64(&self) -> CTypeRef<'a> {
        self.basic_types.int
    }
    pub fn type_f32(&self) -> CTypeRef<'a> {
        self.basic_types.float
    }
    pub fn type_f64(&self) -> CTypeRef<'a> {
        self.basic_types.float
    }
    pub fn type_f80(&self) -> CTypeRef<'a> {
        self.basic_types.float
    }
    pub fn type_void(&self) -> CTypeRef<'a> {
        self.basic_types.void
    }
    pub fn type_str(&self) -> CTypeRef<'a> {
        self.derive_pointer(self.derive_const(self.type_i8()).unwrap())
            .unwrap()
    }

    pub fn construct_from_composed(&self, composed: &CTypeComposed) -> CTypeRef<'a> {
        let base = if composed.has_token(CTypeToken::LongDouble) {
            self.type_f80()
        } else if composed.has_token(CTypeToken::Double) {
            self.type_f64()
        } else if composed.has_token(CTypeToken::Float) {
            self.type_f32()
        } else if composed.has_token(CTypeToken::Void) {
            self.type_void()
        } else if composed.has_token(CTypeToken::Short) {
            if composed.has_token(CTypeToken::Unsigned) {
                self.type_u16()
            } else {
                self.type_i16()
            }
        } else if composed.has_token(CTypeToken::Long) {
            if composed.has_token(CTypeToken::Unsigned) {
                self.type_u64()
            } else {
                self.type_i64()
            }
        } else if composed.has_token(CTypeToken::Char) {
            if composed.has_token(CTypeToken::Unsigned) {
                self.type_u8()
            } else {
                self.type_i8()
            }
        } else {
            if composed.has_token(CTypeToken::Unsigned) {
                self.type_u32()
            } else {
                self.type_i32()
            }
        };
        if composed.has_token(CTypeToken::Const) {
            self.derive_const(base).unwrap()
        } else {
            base
        }
    }
}

pub struct CType<'a> {
    content: CTypeContent<'a>,
    derive: RefCell<CTypeDerive<'a>>,
}

impl Debug for CType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.content {
            CTypeContent::Basic(ref basic) => {
                write!(f, "{:?}", basic)
            }
            CTypeContent::Const(ref base) => {
                write!(f, "{:?} const", base)
            }
            CTypeContent::Pointer(ref base) => {
                write!(f, "{:?}*", base)
            }
            CTypeContent::Array(ref base) => {
                write!(f, "{:?}[N]", base)
            }
            CTypeContent::IncompleteArray(ref base) => {
                write!(f, "{:?}[]", base)
            }
            CTypeContent::Func(ref content) => {
                write!(f, "{:?}", content)
            }
        }
    }
}

impl<'a> CType<'a> {
    fn new_basic(basic: CBasicType) -> Self {
        Self::new(CTypeContent::Basic(basic))
    }

    fn new(content: CTypeContent<'a>) -> Self {
        Self {
            content,
            derive: RefCell::new(CTypeDerive::new()),
        }
    }

    pub fn is_complete(&self) -> bool {
        match self.content {
            CTypeContent::Const(ty) => ty.is_complete(),
            CTypeContent::IncompleteArray(_) | CTypeContent::Basic(CBasicType::Void) => false,
            _ => true,
        }
    }

    pub fn is_complete_ptr(&self) -> bool {
        match self.content {
            CTypeContent::Pointer(ty) => ty.is_complete(),
            CTypeContent::Const(ty) => ty.is_complete_ptr(),
            _ => false,
        }
    }

    pub fn is_array(&self) -> bool {
        match self.content {
            CTypeContent::Array(_) => true,
            CTypeContent::Const(ty) => ty.is_array(),
            _ => false,
        }
    }

    pub fn is_func(&self) -> bool {
        match self.content {
            CTypeContent::Func(_) => true,
            CTypeContent::Const(ty) => ty.is_func(),
            _ => false,
        }
    }

    pub fn is_ptr(&self) -> bool {
        match self.content {
            CTypeContent::Pointer(_) => true,
            CTypeContent::Const(ty) => ty.is_ptr(),
            _ => false,
        }
    }

    pub fn is_integer(&self) -> bool {
        match self.content {
            CTypeContent::Basic(CBasicType::Int) => true,
            CTypeContent::Const(ty) => ty.is_integer(),
            _ => false,
        }
    }

    pub fn is_void(&self) -> bool {
        match self.content {
            CTypeContent::Basic(CBasicType::Void) => true,
            CTypeContent::Const(ty) => ty.is_void(),
            _ => false,
        }
    }

    pub fn is_const(&self) -> bool {
        match self.content {
            CTypeContent::Const(_) => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self.content {
            CTypeContent::Basic(CBasicType::Float) => true,
            CTypeContent::Const(ty) => ty.is_float(),
            _ => false,
        }
    }

    pub fn is_arithmetic(&self) -> bool {
        self.is_integer() || self.is_float()
    }

    pub fn is_scalar(&self) -> bool {
        self.is_arithmetic() || self.is_ptr() || self.is_array()
    }

    pub fn can_self_inc(&self) -> bool {
        match self.content {
            CTypeContent::Basic(CBasicType::Int) => true,
            CTypeContent::Pointer(ptr) => ptr.is_complete(),
            _ => false,
        }
    }

    pub fn get_slice(&self) -> Option<CTypeRef<'a>> {
        match self.content {
            CTypeContent::Pointer(ty) => {
                if ty.is_complete() {
                    Some(ty)
                } else {
                    None
                }
            }
            CTypeContent::Array(ty) => Some(ty),
            CTypeContent::IncompleteArray(ty) => Some(ty),
            _ => None,
        }
    }

    pub fn array_to_ptr(ty: CTypeRef<'a>, pool: &CTypePool<'a>) -> Option<CTypeRef<'a>> {
        match ty.content {
            CTypeContent::Const(ty) => Self::array_to_ptr(ty, pool),
            CTypeContent::Array(ty) => pool.derive_pointer(ty),
            _ => None,
        }
    }

    pub fn remove_pointer_array(&self) -> Option<CTypeRef<'a>> {
        match self.content {
            CTypeContent::Pointer(ty) => Some(ty),
            CTypeContent::Array(ty) => Some(ty),
            CTypeContent::Const(ty) => ty.remove_pointer_array(),
            _ => None,
        }
    }

    pub fn remove_qualifier(ty: CTypeRef<'a>) -> CTypeRef<'a> {
        match ty.content {
            CTypeContent::Const(ty) => ty,
            _ => ty,
        }
    }

    pub fn mult_type(ty1: CTypeRef<'a>, ty2: CTypeRef<'a>) -> Option<CTypeRef<'a>> {
        if ty1.is_integer() && ty2.is_integer() {
            Some(ty1)
        } else if ty1.is_float() && ty2.is_float() {
            Some(ty1)
        } else if ty1.is_integer() && ty2.is_float() {
            Some(ty2)
        } else if ty1.is_float() && ty2.is_integer() {
            Some(ty1)
        } else {
            None
        }
    }

    pub fn mod_type(ty1: CTypeRef<'a>, ty2: CTypeRef<'a>) -> Option<CTypeRef<'a>> {
        if ty1.is_integer() && ty2.is_integer() {
            Some(ty1)
        } else {
            None
        }
    }

    pub fn add_type(
        ty1: CTypeRef<'a>,
        ty2: CTypeRef<'a>,
        pool: &CTypePool<'a>,
    ) -> Option<CTypeRef<'a>> {
        if ty1.is_integer() && ty2.is_integer() {
            Some(ty1)
        } else if ty1.is_float() && ty2.is_float() {
            Some(ty1)
        } else if ty1.is_integer() && ty2.is_float() {
            Some(ty2)
        } else if ty1.is_float() && ty2.is_integer() {
            Some(ty1)
        } else if ty2.is_integer() && ty1.is_complete_ptr() {
            Some(ty1)
        } else if ty1.is_integer() && ty2.is_complete_ptr() {
            Some(ty2)
        } else if ty1.is_integer() && ty2.is_array() {
            CType::array_to_ptr(ty2, pool)
        } else if ty2.is_integer() && ty1.is_array() {
            CType::array_to_ptr(ty1, pool)
        } else {
            None
        }
    }

    pub fn subtract_type(
        ty1: CTypeRef<'a>,
        ty2: CTypeRef<'a>,
        pool: &CTypePool<'a>,
    ) -> Option<CTypeRef<'a>> {
        if ty1.is_integer() && ty2.is_integer() {
            Some(ty1)
        } else if ty1.is_float() && ty2.is_float() {
            Some(ty1)
        } else if ty1.is_integer() && ty2.is_float() {
            Some(ty2)
        } else if ty1.is_float() && ty2.is_integer() {
            Some(ty1)
        } else if let (Some(ty1r), Some(ty2r)) =
            (ty1.remove_pointer_array(), ty2.remove_pointer_array())
        {
            if CType::remove_qualifier(ty1r) == CType::remove_qualifier(ty2r) {
                Some(pool.type_i64())
            } else {
                None
            }
        } else if ty1.is_complete_ptr() && ty2.is_integer() {
            Some(ty1)
        } else if ty1.is_array() && ty2.is_integer() {
            CType::array_to_ptr(ty1, pool)
        } else {
            None
        }
    }

    pub fn shift_type(ty1: CTypeRef<'a>, ty2: CTypeRef<'a>) -> Option<CTypeRef<'a>> {
        if ty1.is_integer() && ty2.is_integer() {
            Some(ty1)
        } else {
            None
        }
    }

    pub fn compare_type(
        ty1: CTypeRef<'a>,
        ty2: CTypeRef<'a>,
        pool: &CTypePool<'a>,
    ) -> Option<CTypeRef<'a>> {
        if ty1.is_arithmetic() && ty2.is_arithmetic() {
            Some(pool.type_i32())
        } else if let (Some(ty1r), Some(ty2r)) =
            (ty1.remove_pointer_array(), ty2.remove_pointer_array())
        {
            if CType::remove_qualifier(ty1r) == CType::remove_qualifier(ty2r) {
                Some(pool.type_i32())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn equals_type(
        ty1: CTypeRef<'a>,
        ty2: CTypeRef<'a>,
        pool: &CTypePool<'a>,
    ) -> Option<CTypeRef<'a>> {
        if (ty1.is_integer() || ty1.is_ptr() || ty1.is_array())
            && (ty2.is_integer() || ty2.is_ptr() || ty2.is_array())
        {
            Some(pool.type_i32())
        } else if ty1.is_arithmetic() && ty2.is_arithmetic() {
            Some(pool.type_i32())
        } else {
            None
        }
    }

    pub fn bitop_type(ty1: CTypeRef<'a>, ty2: CTypeRef<'a>) -> Option<CTypeRef<'a>> {
        if ty1.is_integer() && ty2.is_integer() {
            Some(ty1)
        } else {
            None
        }
    }

    pub fn logicop_type(
        ty1: CTypeRef<'a>,
        ty2: CTypeRef<'a>,
        pool: &CTypePool<'a>,
    ) -> Option<CTypeRef<'a>> {
        if ty1.is_scalar() && ty2.is_scalar() {
            Some(pool.type_i32())
        } else {
            None
        }
    }

    pub fn assign_type(ty1: CTypeRef<'a>, ty2: CTypeRef<'a>) -> Option<CTypeRef<'a>> {
        if ty1.is_const() || ty1.is_array() || ty1.is_func() {
            None
        } else if CType::can_cast_to(ty2, ty1) {
            Some(ty1)
        } else {
            None
        }
    }

    pub fn plusassign_type(ty1: CTypeRef<'a>, ty2: CTypeRef<'a>) -> Option<CTypeRef<'a>> {
        if ty1.is_const() || ty1.is_array() {
            None
        } else {
            if ty1.is_scalar() && ty2.is_scalar() {
                Some(ty1)
            } else if ty1.is_complete_ptr() && ty2.is_integer() {
                Some(ty1)
            } else {
                None
            }
        }
    }

    pub fn condition_type(ty1: CTypeRef<'a>, ty2: CTypeRef<'a>) -> Option<CTypeRef<'a>> {
        let ty1 = CType::remove_qualifier(ty1);
        let ty2 = CType::remove_qualifier(ty2);

        if ty1 == ty2 {
            Some(ty1)
        } else if ty1.is_arithmetic() && ty2.is_arithmetic() {
            Some(ty1)
        } else if ty1.is_void() && ty2.is_void() {
            Some(ty1)
        } else {
            None
        }
    }

    pub fn can_cast_to(from: CTypeRef<'a>, to: CTypeRef<'a>) -> bool {
        (CType::remove_qualifier(from) == CType::remove_qualifier(to))
            || to.is_void()
            || ((from.is_integer() || from.is_ptr()) && (to.is_integer() || to.is_ptr()))
            || (from.is_arithmetic() && to.is_arithmetic())
    }
}

pub struct CTypeDerive<'a> {
    derive_pointer: Option<CTypeRef<'a>>,
    derive_const: Option<CTypeRef<'a>>,
    derive_array: Option<CTypeRef<'a>>,
    derive_imcomplete_array: Option<CTypeRef<'a>>,
    derive_func: HashMap<(bool, &'a [CTypeRef<'a>]), CTypeRef<'a>>,
}

impl<'a> CTypeDerive<'a> {
    fn new() -> Self {
        Self {
            derive_pointer: None,
            derive_const: None,
            derive_array: None,
            derive_imcomplete_array: None,
            derive_func: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub enum CTypeContent<'a> {
    Basic(CBasicType),
    Const(CTypeRef<'a>),
    Pointer(CTypeRef<'a>),
    Array(CTypeRef<'a>),
    IncompleteArray(CTypeRef<'a>),
    Func(CFuncContent<'a>),
}

#[derive(Debug, Getters, CopyGetters)]
pub struct CFuncContent<'a> {
    #[get_copy = "pub"]
    ret: CTypeRef<'a>,
    #[get = "pub"]
    args: Vec<CTypeRef<'a>>,
    #[get_copy = "pub"]
    va_args: bool,
}

pub type CFuncContentRef<'a> = Pointer<'a, CFuncContent<'a>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Prop)]
pub struct CExprInfo<'a> {
    pub ty: CTypeRef<'a>,
    pub is_lvalue: bool,
}

impl<'a> CExprInfo<'a> {
    pub fn new(mut ty: CTypeRef<'a>, is_lvalue: bool) -> Self {
        if !is_lvalue {
            ty = CType::remove_qualifier(ty);
        }
        Self { ty, is_lvalue }
    }
}
