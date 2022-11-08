//! A tiny crate that provides `#[derive(Deref)]` and `#[derive(DerefMut)]`.
//!
//! While this in unidiomatic, it can be useful and sees widespread use in the community.
//! Therefore, this crate provides a macro to derive [`Deref`](std::ops::Deref)
//! and [`DerefMut`](std::ops::DerefMut) for you to help reduce boilerplate.

/// Used to derive [`Deref`](std::ops::Deref) for a struct.
///
/// # Example
/// If have a struct with only one field, you can derive `Deref` for it.
/// ```rust
/// # use deref_derive::Deref;
/// #[derive(Default, Deref)]
/// struct Foo {
///     field: String,
/// }
///
/// assert_eq!(Foo::default().len(), 0);
/// ```
/// If you have a struct with multiple fields, you will have to use the `deref` attribute.
/// ```rust
/// # use deref_derive::Deref;
/// #[derive(Default, Deref)]
/// struct Foo {
///    #[deref]
///    field: u32,
///    other_field: String,
/// }
///
/// assert_eq!(*Foo::default(), 0);
/// ```
/// Tuple structs are also supported.
/// ```rust
/// # use deref_derive::{Deref, DerefMut};
/// #[derive(Default, Deref, DerefMut)]
/// struct Foo(u32, #[deref] String);
///
/// let mut foo = Foo::default();
/// *foo = "bar".to_string();
/// foo.push('!');
///
/// assert_eq!(*foo, "bar!");
#[proc_macro_derive(Deref, attributes(deref))]
pub fn derive_deref(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let ident = input.ident;

    let target = DerefTarget::get(&input.data);
    let target_ty = target.ty;
    let target_field = target.field;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote::quote! {
        #[automatically_derived]
        impl #impl_generics ::std::ops::Deref for #ident #ty_generics #where_clause {
            type Target = #target_ty;

            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.#target_field
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

/// Used to derive [`DerefMut`](std::ops::DerefMut) for a struct.
///
/// For examples, see [`Deref`].
#[proc_macro_derive(DerefMut, attributes(deref))]
pub fn derive_deref_mut(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let ident = input.ident;

    let target = DerefTarget::get(&input.data);
    let target_field = target.field;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote::quote! {
        #[automatically_derived]
        impl #impl_generics ::std::ops::DerefMut for #ident #ty_generics #where_clause {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.#target_field
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

struct DerefTarget {
    ty: syn::Type,
    field: proc_macro2::TokenStream,
    has_attr: bool,
}

impl DerefTarget {
    const ATTR_NAME: &'static str = "deref";

    fn has_attr(attrs: &[syn::Attribute]) -> bool {
        attrs.iter().any(|attr| attr.path.is_ident(Self::ATTR_NAME))
    }

    fn get_target(mut targets: impl ExactSizeIterator<Item = Self>) -> Self {
        if targets.len() == 1 {
            targets.next().unwrap()
        } else {
            let targets = targets.filter(|target| target.has_attr).collect::<Vec<_>>();

            if targets.len() == 1 {
                targets.into_iter().next().unwrap()
            } else {
                panic!("expected exactly one field with #[deref] attribute");
            }
        }
    }

    fn get(data: &syn::Data) -> Self {
        match data {
            syn::Data::Struct(data) => match data.fields {
                syn::Fields::Named(ref fields) => {
                    let fields = fields.named.iter().map(|f| {
                        let ty = f.ty.clone();
                        let field = f.ident.clone().unwrap();
                        let has_attr = Self::has_attr(&f.attrs);

                        Self {
                            ty,
                            field: quote::quote!(#field),
                            has_attr,
                        }
                    });

                    Self::get_target(fields)
                }
                syn::Fields::Unnamed(ref fields) => {
                    let fields = fields.unnamed.iter().enumerate().map(|(i, f)| {
                        let ty = f.ty.clone();
                        let field = syn::Index::from(i);
                        let has_attr = Self::has_attr(&f.attrs);

                        Self {
                            ty,
                            field: quote::quote!(#field),
                            has_attr,
                        }
                    });

                    Self::get_target(fields)
                }
                syn::Fields::Unit => {
                    panic!("cannot be derived for unit structs")
                }
            },
            _ => unimplemented!("can only be derived for structs"),
        }
    }
}
