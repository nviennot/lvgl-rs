use inflector::cases::pascalcase::to_pascal_case;
use lazy_static::lazy_static;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use quote::{format_ident, ToTokens};
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use syn::{FnArg, ForeignItem, ForeignItemFn, Item, ItemFn, ReturnType};

type CGResult<T> = Result<T, Box<dyn Error>>;

const LIB_PREFIX: &str = "lv_";

lazy_static! {
    static ref TYPE_MAPPINGS: HashMap<&'static str, &'static str> = [
        ("i16", "i16"),
        ("u16", "u16"),
        ("i32", "i32"),
        ("u8", "u8"),
        ("u32", "u32"),
        ("bool", "bool"),

        ("lv_opa_t", "lv_opa_t"),
        ("lv_anim_enable_t", "lv_anim_enable_t"),
        ("lv_arc_mode_t", "lv_arc_mode_t"),
        ("lv_bar_mode_t", "lv_bar_mode_t"),
        ("lv_btnmatrix_ctrl_t", "lv_btnmatrix_ctrl_t"),
        ("lv_chart_axis_t", "lv_chart_axis_t"),
        ("lv_chart_type_t", "lv_chart_type_t"),
        ("lv_chart_update_mode_t", "lv_chart_update_mode_t"),
        ("lv_color_t", "lv_color_t"),
        ("lv_coord_t", "lv_coord_t"),
        ("lv_dir_t", "lv_dir_t"),
        ("lv_img_size_mode_t", "lv_img_size_mode_t"),
        ("lv_imgbtn_state_t", "lv_imgbtn_state_t"),
        ("lv_keyboard_mode_t", "lv_keyboard_mode_t"),
        ("lv_label_long_mode_t", "lv_label_long_mode_t"),
        ("lv_menu_mode_header_t", "lv_menu_mode_header_t"),
        ("lv_menu_mode_root_back_btn_t", "lv_menu_mode_root_back_btn_t"),
        ("lv_roller_mode_t", "lv_roller_mode_t"),
        ("lv_slider_mode_t", "lv_slider_mode_t"),
        ("lv_span_mode_t", "lv_span_mode_t"),
        ("lv_span_overflow_t", "lv_span_overflow_t"),
        ("lv_table_cell_ctrl_t", "lv_table_cell_ctrl_t"),
        ("lv_text_align_t", "lv_text_align_t"),


        ("lv_coord_t", "lv_coord_t"),
        ("* const cty :: c_char", "_"),
    ]
    .iter()
    .cloned()
    .collect();
}

#[derive(Debug, Copy, Clone)]
pub enum WrapperError {
    Skip,
}

pub type WrapperResult<T> = Result<T, WrapperError>;

pub trait Rusty {
    type Parent;

    fn code(&self, parent: &Self::Parent) -> WrapperResult<TokenStream>;
}

#[derive(Debug, Clone)]
pub struct LvWidget {
    name: String,
    methods: Vec<LvFunc>,
}

impl Rusty for LvWidget {
    type Parent = ();

    fn code(&self, _parent: &Self::Parent) -> WrapperResult<TokenStream> {
        // We don't generate for the generic Obj
        if self.name.as_str().eq("obj") {
            return Err(WrapperError::Skip);
        }

        let widget_name = format_ident!("{}", to_pascal_case(self.name.as_str()));
        let methods: Vec<TokenStream> = self.methods.iter().flat_map(|m| m.code(self)).collect();

        Ok(quote! {
            define_object!(#widget_name);

            impl<C: 'static> #widget_name<C> {
                #(#methods)*
            }
        })
    }
}

#[derive(Clone, Debug)]
pub struct LvFunc {
    name: String,
    args: Vec<LvArg>,
    ret: Option<LvType>,
}

impl LvFunc {
    pub fn new(name: String, args: Vec<LvArg>, ret: Option<LvType>) -> Self {
        Self { name, args, ret }
    }

    pub fn is_method(&self) -> bool {
        if !self.args.is_empty() {
            let first_arg = &self.args[0];
            return first_arg.typ.literal_name.contains("lv_obj_t");
        }
        false
    }
}

impl Rusty for LvFunc {
    type Parent = LvWidget;

    fn code(&self, parent: &Self::Parent) -> WrapperResult<TokenStream> {
        let templ = format!("{}{}_", LIB_PREFIX, parent.name.as_str());
        let mut new_name = self.name.replace(templ.as_str(), "");
        let original_func_name = format_ident!("{}", self.name.as_str());

        // generate constructor
        if new_name.as_str().eq("create") {
            return Ok(quote! {
                pub fn new<'a>(parent: &mut impl crate::core::ObjExt<C>) -> Self {
                    unsafe {
                        let obj = lvgl_sys::#original_func_name(&mut *parent.raw);
                        let obj = Obj::from_raw(obj.as_mut().expect("OOM"), parent.context);
                        Self { obj }
                    }
                }
            });
        }

        if let Some(n) = new_name.strip_prefix("set_") {
            new_name = n.to_string();
            if new_name == "type" {
                new_name = "type_".to_string();
            }
        }

        let func_name = format_ident!("{}", new_name);

        // We don't deal with methods that return types yet
        if self.ret.is_some() {
            return Err(WrapperError::Skip);
        }

        // Make sure all arguments can be generated, skip the first arg (self)!
        for arg in self.args.iter().skip(1) {
            arg.code(self)?;
        }

        let args_decl = self
            .args
            .iter()
            .enumerate()
            .fold(quote!(), |args, (i, arg)| {
                // if first arg is `const`, then it should be immutable
                let next_arg = if i == 0 {
                    if arg.get_type().is_const() {
                        quote!(self)
                    } else {
                        quote!(mut self)
                    }
                } else {
                    arg.code(self).unwrap()
                };
                if args.is_empty() {
                    quote! {
                        #next_arg
                    }
                } else {
                    quote! {
                        #args, #next_arg
                    }
                }
            });

        let args_processing = self
            .args
            .iter()
            .enumerate()
            .fold(quote!(), |args, (i, arg)| {
                // if first arg is `const`, then it should be immutable
                let next_arg = if i == 0 {
                    quote!()
                } else {
                    let var = arg.get_processing();
                    quote!(#var)
                };
                if args.is_empty() {
                    quote! {
                        #next_arg
                    }
                } else {
                    quote! {
                        #args
                        #next_arg
                    }
                }
            });

        let args_call = self
            .args
            .iter()
            .enumerate()
            .fold(quote!(), |args, (i, arg)| {
                // if first arg is `const`, then it should be immutable
                let next_arg = if i == 0 {
                    quote!(&mut *self.raw)
                } else {
                    let var = arg.get_value_usage();
                    quote!(#var)
                };
                if args.is_empty() {
                    quote! {
                        #next_arg
                    }
                } else {
                    quote! {
                        #args, #next_arg
                    }
                }
            });

        // TODO: Handle methods that return types
        Ok(quote! {
            pub fn #func_name(#args_decl) -> Self {
                #args_processing
                unsafe {
                    lvgl_sys::#original_func_name(#args_call);
                }
                self
            }
        })
    }
}

impl From<&ForeignItemFn> for LvFunc {
    fn from(ffi: &ForeignItemFn) -> Self {
        let ret = match &ffi.sig.output {
            ReturnType::Default => None,
            ReturnType::Type(_, typ) => Some(typ.clone().into()),
        };
        Self::new(
            ffi.sig.ident.to_string(),
            ffi.sig
                .inputs
                .iter()
                .filter_map(|fa| {
                    // Since we know those are foreign functions, we only care about typed arguments
                    if let FnArg::Typed(tya) = fa {
                        Some(tya)
                    } else {
                        None
                    }
                })
                .map(|a| a.clone().into())
                .collect::<Vec<LvArg>>(),
            ret,
        )
    }
}

// Copy/paste of From<&ForeignItemFn> for LvFunc
impl From<&ItemFn> for LvFunc {
    fn from(ffi: &ItemFn) -> Self {
        let ret = match &ffi.sig.output {
            ReturnType::Default => None,
            ReturnType::Type(_, typ) => Some(typ.clone().into()),
        };
        Self::new(
            ffi.sig.ident.to_string(),
            ffi.sig
                .inputs
                .iter()
                .filter_map(|fa| {
                    // Since we know those are foreign functions, we only care about typed arguments
                    if let FnArg::Typed(tya) = fa {
                        Some(tya)
                    } else {
                        None
                    }
                })
                .map(|a| a.clone().into())
                .collect::<Vec<LvArg>>(),
            ret,
        )
    }
}


#[derive(Clone, Debug)]
pub struct LvArg {
    name: String,
    typ: LvType,
}

impl From<syn::PatType> for LvArg {
    fn from(fa: syn::PatType) -> Self {
        Self::new(fa.pat.to_token_stream().to_string(), fa.ty.into())
    }
}

impl LvArg {
    pub fn new(name: String, typ: LvType) -> Self {
        Self { name, typ }
    }

    pub fn get_name_ident(&self) -> Ident {
        // Filter Rust language keywords
        syn::parse_str::<syn::Ident>(self.name.as_str())
            .unwrap_or_else(|_| format_ident!("r#{}", self.name.as_str()))
    }

    pub fn get_processing(&self) -> TokenStream {
        // TODO: A better way to handle this, instead of `is_sometype()`, is using the Rust
        //       type system itself.

        // No need to pre-process this type of argument
        quote! {}
    }

    pub fn get_value_usage(&self) -> TokenStream {
        let ident = self.get_name_ident();
        if self.typ.is_str() {
            quote! {
                #ident.as_ptr()
            }
        } else {
            quote! {
                #ident
            }
        }
    }

    pub fn get_type(&self) -> &LvType {
        &self.typ
    }
}

impl Rusty for LvArg {
    type Parent = LvFunc;

    fn code(&self, _parent: &Self::Parent) -> WrapperResult<TokenStream> {
        let name = self.get_name_ident();
        let typ = self.typ.code(self)?;
        Ok(quote! {
            #name: #typ
        })
    }
}

#[derive(Clone, Debug)]
pub struct LvType {
    literal_name: String,
}

impl LvType {
    pub fn new(literal_name: String) -> Self {
        Self {
            literal_name,
        }
    }

    pub fn from(r_type: Box<syn::Type>) -> Self {
        Self {
            literal_name: r_type.to_token_stream().to_string(),
        }
    }

    pub fn is_const(&self) -> bool {
        self.literal_name.starts_with("const ")
    }

    pub fn is_str(&self) -> bool {
        self.literal_name.ends_with("* const cty :: c_char")
    }
}

impl Rusty for LvType {
    type Parent = LvArg;

    fn code(&self, _parent: &Self::Parent) -> WrapperResult<TokenStream> {
        match TYPE_MAPPINGS.get(self.literal_name.as_str()) {
            Some(name) => {
                let val = if self.is_str() {
                    quote!(&cstr_core::CStr)
                } else {
                    let ident = format_ident!("{}", name);
                    if name.ends_with("_t") {
                        quote!(lvgl_sys::#ident)
                    } else {
                        quote!(#ident)
                    }
                };
                Ok(quote! {
                    #val
                })
            }
            None => {
                eprintln!("Skipping type {}", self.literal_name);
                Err(WrapperError::Skip)
            }
        }
    }
}

impl From<Box<syn::Type>> for LvType {
    fn from(t: Box<syn::Type>) -> Self {
        Self::from(t)
    }
}

pub struct CodeGen {
    functions: Vec<LvFunc>,
    widgets: Vec<LvWidget>,
}

impl CodeGen {
    pub fn from(code: &str) -> CGResult<Self> {
        let functions = Self::load_func_defs(code)?;
        let widgets = Self::extract_widgets(&functions)?;
        Ok(Self { functions, widgets })
    }

    pub fn get_widgets(&self) -> &Vec<LvWidget> {
        &self.widgets
    }

    fn extract_widgets(functions: &[LvFunc]) -> CGResult<Vec<LvWidget>> {
        let widget_names = Self::get_widget_names(functions);

        let mut widgets = widget_names.iter().map(|n|
            (n.as_str(), LvWidget {
            name: n.clone(),
            methods: Vec::new(),
        })).collect::<HashMap<_,_>>();

        for f in functions {
            if !f.is_method() {
                continue
            }

            if let Some((widget_name, _)) = f.name.strip_prefix(LIB_PREFIX).unwrap_or("")
                .split_once("_") {

                    if let Some(entry) = widgets.get_mut(widget_name) {
                        entry.methods.push(f.clone())
                    }
            }
        }

        Ok(widgets.values().cloned().collect())
    }

    fn get_widget_names(functions: &[LvFunc]) -> Vec<String> {
        let reg = format!("^{}([^_]+)_create$", LIB_PREFIX);
        let create_func = Regex::new(reg.as_str()).unwrap();

        functions
            .iter()
            .filter(|e| create_func.is_match(e.name.as_str()) && e.args.len() == 1)
            .map(|f| {
                String::from(
                    create_func
                        .captures(f.name.as_str())
                        .unwrap()
                        .get(1)
                        .unwrap()
                        .as_str(),
                )
            })
            .collect::<Vec<_>>()
    }

    pub fn load_func_defs(bindgen_code: &str) -> CGResult<Vec<LvFunc>> {
        let ast: syn::File = syn::parse_str(bindgen_code)?;
        let mut fns = ast
            .items
            .iter()
            .filter_map(|e| {
                if let Item::ForeignMod(fm) = e {
                    Some(fm)
                } else {
                    None
                }
            })
            .flat_map(|e| {
                e.items.iter().filter_map(|it| {
                    if let ForeignItem::Fn(f) = it {
                        Some(f)
                    } else {
                        None
                    }
                })
            })
            .filter(|ff| ff.sig.ident.to_string().starts_with(LIB_PREFIX))
            .map(|ff| ff.into())
            .collect::<Vec<LvFunc>>();

        let mut fns2 = ast
            .items
            .iter()
            .filter_map(|e| {
                if let Item::Fn(fm) = e {
                    Some(fm)
                } else {
                    None
                }
            })
            .filter(|ff| ff.sig.ident.to_string().starts_with(LIB_PREFIX))
            .map(|ff| ff.into())
            .collect::<Vec<LvFunc>>();

        fns.append(&mut fns2);
        Ok(fns)
    }

    pub fn get_function_names(&self) -> CGResult<Vec<String>> {
        Ok(self.functions.iter().map(|f| f.name.clone()).collect())
    }
}

#[cfg(test)]
mod test {
    use crate::{CodeGen, LvArg, LvFunc, LvType, LvWidget, Rusty};
    use quote::quote;

    #[test]
    fn can_load_bindgen_fns() {
        let bindgen_code = quote! {
            extern "C" {
                #[doc = " Return with the screen of an object"]
                #[doc = " @param obj pointer to an object"]
                #[doc = " @return pointer to a screen"]
                pub fn lv_obj_get_screen(obj: *const lv_obj_t) -> *mut lv_obj_t;
            }
        };

        let cg = CodeGen::load_func_defs(bindgen_code.to_string().as_str()).unwrap();

        let ffn = cg.get(0).unwrap();
        assert_eq!(ffn.name, "lv_obj_get_screen");
        assert_eq!(ffn.args[0].name, "obj");
    }

    #[test]
    fn can_identify_widgets_from_function_names() {
        let funcs = vec![
            LvFunc::new(
                "lv_obj_create".to_string(),
                vec![
                    LvArg::new("parent".to_string(), LvType::new("abc".to_string())),
                    LvArg::new("copy_from".to_string(), LvType::new("bcf".to_string())),
                ],
                None,
            ),
            LvFunc::new(
                "lv_btn_create".to_string(),
                vec![
                    LvArg::new("parent".to_string(), LvType::new("abc".to_string())),
                    LvArg::new("copy_from".to_string(), LvType::new("bcf".to_string())),
                ],
                None,
            ),
            LvFunc::new(
                "lv_do_something".to_string(),
                vec![
                    LvArg::new("parent".to_string(), LvType::new("abc".to_string())),
                    LvArg::new("copy_from".to_string(), LvType::new("bcf".to_string())),
                ],
                None,
            ),
            LvFunc::new(
                "lv_invalid_create".to_string(),
                vec![LvArg::new(
                    "parent".to_string(),
                    LvType::new("abc".to_string()),
                )],
                None,
            ),
            LvFunc::new(
                "lv_cb_create".to_string(),
                vec![
                    LvArg::new("parent".to_string(), LvType::new("abc".to_string())),
                    LvArg::new("copy_from".to_string(), LvType::new("bcf".to_string())),
                ],
                None,
            ),
        ];

        let widget_names = CodeGen::get_widget_names(&funcs);

        assert_eq!(widget_names.len(), 3);
    }

    #[test]
    fn generate_method_wrapper() {
        // pub fn lv_arc_set_bg_end_angle(arc: *mut lv_obj_t, end: u16);
        let arc_set_bg_end_angle = LvFunc::new(
            "lv_arc_set_bg_end_angle".to_string(),
            vec![
                LvArg::new("arc".to_string(), LvType::new("*mut lv_obj_t".to_string())),
                LvArg::new("end".to_string(), LvType::new("u16".to_string())),
            ],
            None,
        );
        let arc_widget = LvWidget {
            name: "arc".to_string(),
            methods: vec![],
        };

        let code = arc_set_bg_end_angle.code(&arc_widget).unwrap();
        let expected_code = quote! {
            pub fn set_bg_end_angle(&mut self, end: u16) -> crate::LvResult<()> {
                unsafe {
                    lvgl_sys::lv_arc_set_bg_end_angle(self.core.raw().as_mut(), end);
                }
                Ok(())
            }
        };

        assert_eq!(code.to_string(), expected_code.to_string());
    }

    #[test]
    fn generate_method_wrapper_for_str_types_as_argument() {
        let bindgen_code = quote! {
            extern "C" {
                #[doc = " Set a new text for a label. Memory will be allocated to store the text by the label."]
                #[doc = " @param label pointer to a label object"]
                #[doc = " @param text '\\0' terminated character string. NULL to refresh with the current text."]
                pub fn lv_label_set_text(label: *mut lv_obj_t, text: *const cty::c_char);
            }
        };
        let cg = CodeGen::load_func_defs(bindgen_code.to_string().as_str()).unwrap();

        let label_set_text = cg.get(0).unwrap().clone();
        let parent_widget = LvWidget {
            name: "label".to_string(),
            methods: vec![],
        };

        let code = label_set_text.code(&parent_widget).unwrap();
        let expected_code = quote! {

            pub fn set_text(&mut self, text: &cstr_core::CStr) -> crate::LvResult<()> {
                unsafe {
                    lvgl_sys::lv_label_set_text(
                        self.core.raw().as_mut(),
                        text.as_ptr()
                    );
                }
                Ok(())
            }

        };

        assert_eq!(code.to_string(), expected_code.to_string());
    }

    #[test]
    fn generate_basic_widget_code() {
        let arc_widget = LvWidget {
            name: "arc".to_string(),
            methods: vec![],
        };

        let code = arc_widget.code(&()).unwrap();
        let expected_code = quote! {
            define_object!(Arc);

            impl Arc {

            }
        };

        assert_eq!(code.to_string(), expected_code.to_string());
    }

    #[test]
    fn generate_widget_with_constructor_code() {
        // pub fn lv_arc_create(par: *mut lv_obj_t, copy: *const lv_obj_t) -> *mut lv_obj_t;
        let arc_create = LvFunc::new(
            "lv_arc_create".to_string(),
            vec![
                LvArg::new("par".to_string(), LvType::new("*mut lv_obj_t".to_string())),
                LvArg::new(
                    "copy".to_string(),
                    LvType::new("*const lv_obj_t".to_string()),
                ),
            ],
            Some(LvType::new("*mut lv_obj_t".to_string())),
        );

        let arc_widget = LvWidget {
            name: "arc".to_string(),
            methods: vec![arc_create],
        };

        let code = arc_widget.code(&()).unwrap();
        let expected_code = quote! {
            define_object!(Arc);

            impl Arc {
                pub fn new<C>(parent: &mut C) -> crate::LvResult<Self>
                where
                    C: crate::core::NativeObject,
                {

                    unsafe {
                        let ptr = lvgl_sys::lv_arc_create(parent.raw().as_mut());
                        let raw = core::ptr::NonNull::new(ptr).except("OOM");
                        let core = <crate::Obj as crate::Widget>::from_raw(raw);
                        Ok(Self { core })
                    }
                }
            }
        };

        assert_eq!(code.to_string(), expected_code.to_string());
    }
}
