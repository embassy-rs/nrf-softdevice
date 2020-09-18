#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

use darling::FromMeta;
use proc_macro::{TokenStream};
use proc_macro2::{TokenStream as TokenStream2};
use quote::{format_ident, quote};
use std::iter::FromIterator;
use syn::spanned::Spanned;

#[derive(Debug, FromMeta)]
struct ServerArgs {
    uuid: String,
}
#[derive(Debug, FromMeta)]
struct CharacteristicArgs {
    uuid: String,
    #[darling(default)]
    read: bool,
    #[darling(default)]
    write: bool,
    #[darling(default)]
    notify: bool,
    #[darling(default)]
    indicate: bool,
}

#[derive(Debug)]
struct Characteristic {
    name: String,
    ty: syn::Type,
    args: CharacteristicArgs,
}

#[proc_macro_attribute]
pub fn gatt_server(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let mut struc = syn::parse_macro_input!(item as syn::ItemStruct);

    let _args = match ServerArgs::from_list(&args) {
        Ok(v) => v,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    let mut chars = Vec::new();

    let struct_fields = match &mut struc.fields {
        syn::Fields::Named(n) => n,
        _ => {
            struc
                .ident
                .span()
                .unwrap()
                .error("gatt_server structs must have named fields, not tuples.")
                .emit();
            return TokenStream::new();
        }
    };
    let mut fields = struct_fields
        .named
        .iter()
        .cloned()
        .collect::<Vec<syn::Field>>();
    let mut err = None;
    fields.retain(|field| {
        if let Some(attr) = field.attrs.iter().find(|attr| {
            attr.path.segments.len() == 1
                && attr.path.segments.first().unwrap().ident.to_string() == "characteristic"
        }) {
            let args = attr.parse_meta().unwrap();

            let args = match CharacteristicArgs::from_meta(&args) {
                Ok(v) => v,
                Err(e) => {
                    err = Some(e.write_errors().into());
                    return false;
                }
            };

            chars.push(Characteristic {
                name: field.ident.as_ref().unwrap().to_string(),
                ty: field.ty.clone(),
                args,
            });

            false
        } else {
            true
        }
    });

    if let Some(err) = err {
        return err;
    }

    //panic!("chars {:?}", chars);
    let struct_name = struc.ident.clone();
    let event_enum_name = format_ident!("{}Event", struct_name);

    let mut code_impl = TokenStream2::new();
    let mut code_register_chars = TokenStream2::new();
    let mut code_register_init = TokenStream2::new();
    let mut code_on_write = TokenStream2::new();
    let mut code_event_enum = TokenStream2::new();

    for ch in &chars {
        let name_pascal = inflector::cases::pascalcase::to_pascal_case(&ch.name);
        let char_name = format_ident!("{}", ch.name);
        let value_handle = format_ident!("{}_value_handle", ch.name);
        let cccd_handle = format_ident!("{}_cccd_handle", ch.name);
        let get_fn = format_ident!("{}_get", ch.name);
        let set_fn = format_ident!("{}_set", ch.name);
        let notify_fn = format_ident!("{}_notify", ch.name);

        let read = ch.args.read;
        let write = ch.args.write;
        let notify = ch.args.notify;
        let indicate = ch.args.indicate;
        let ty = &ch.ty;

        fields.push(syn::Field {
            ident: Some(value_handle.clone()),
            ty: syn::Type::Verbatim(quote!(u16).into()),
            attrs: Vec::new(),
            colon_token: Default::default(),
            vis: syn::Visibility::Inherited,
        });

        code_register_chars.extend(quote!(
            let #char_name = register_char(
                Characteristic {
                    uuid: GATT_BAS_BATTERY_LEVEL_CHAR_UUID,
                    can_read: #read,
                    can_write: #write,
                    can_notify: #notify,
                    can_indicate: #indicate,
                    max_len: 1,
                },
                &[123],
            )?;
        ));

        code_register_init.extend(quote!(
            #value_handle: #char_name.value_handle,
        ));

        code_impl.extend(quote!(
            fn #get_fn(&self) -> Result<u8, gatt_server::GetValueError> {
                let sd = unsafe { ::nrf_softdevice::Softdevice::steal() };
                let buf = &mut [0u8; 1];
                gatt_server::get_value(sd, self.#value_handle, buf)?;
                Ok(buf[0])
            }

            fn #set_fn(&self, val: u8) -> Result<(), gatt_server::SetValueError> {
                let sd = unsafe { ::nrf_softdevice::Softdevice::steal() };
                gatt_server::set_value(sd, self.#value_handle, &[val])
            }
        ));

        if ch.args.indicate || ch.args.notify {
            fields.push(syn::Field {
                ident: Some(cccd_handle.clone()),
                ty: syn::Type::Verbatim(quote!(u16).into()),
                attrs: Vec::new(),
                colon_token: Default::default(),
                vis: syn::Visibility::Inherited,
            });
            code_register_init.extend(quote!(
                #cccd_handle: #char_name.cccd_handle,
            ));
        }

        if ch.args.write {
            let case_write = format_ident!("{}Write", name_pascal);
            code_event_enum.extend(quote!(
                #case_write(#ty),
            ));
            #[rustfmt::skip]
            code_on_write.extend(quote!(
                if handle == self.#value_handle {
                    return Some(#event_enum_name::#case_write(data[0]));
                }
            ));
        }
        if ch.args.notify {
            let case_enabled = format_ident!("{}NotificationsEnabled", name_pascal);
            let case_disabled = format_ident!("{}NotificationsDisabled", name_pascal);

            code_impl.extend(quote!(
                fn #notify_fn(
                    &self,
                    conn: &Connection,
                    val: u8,
                ) -> Result<(), gatt_server::NotifyValueError> {
                    gatt_server::notify_value(conn, self.#value_handle, &[val])
                }
            ));

            code_event_enum.extend(quote!(
                #case_enabled,
                #case_disabled,
            ));
            #[rustfmt::skip]
            code_on_write.extend(quote!(

                if handle == self.#cccd_handle {
                    if data.len() != 0 && data[0] & 0x01 != 0 {
                        return Some(#event_enum_name::#case_enabled);
                    } else {
                        return Some(#event_enum_name::#case_disabled);
                    }
                }
            ));
        }

        //panic!();
    }

    struct_fields.named = syn::punctuated::Punctuated::from_iter(fields);

    let result = quote! {
        #struc

        impl #struct_name {
            #code_impl
        }

        impl ::nrf_softdevice::ble::gatt_server::Server for #struct_name {
            type Event = #event_enum_name;

            fn uuid() -> Uuid {
                GATT_BAS_SVC_UUID
            }

            fn register<F>(service_handle: u16, mut register_char: F) -> Result<Self, RegisterError>
            where
                F: FnMut(Characteristic, &[u8]) -> Result<CharacteristicHandles, RegisterError>,
            {
                #code_register_chars

                Ok(Self {
                    #code_register_init
                })
            }

            fn on_write(&self, handle: u16, data: &[u8]) -> Option<Self::Event> {
                #code_on_write
                None
            }

        }

        enum #event_enum_name {
            #code_event_enum
        }
    };
    result.into()
}
