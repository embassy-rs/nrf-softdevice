#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, quote_spanned};
use std::iter::FromIterator;
use syn::spanned::Spanned;

mod uuid;

use crate::uuid::Uuid;

#[derive(Debug, FromMeta)]
struct ServerArgs {
    uuid: Uuid,
}
#[derive(Debug, FromMeta)]
struct CharacteristicArgs {
    uuid: Uuid,
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
    span: Span,
}

#[proc_macro_attribute]
pub fn gatt_server(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let mut struc = syn::parse_macro_input!(item as syn::ItemStruct);

    let args = match ServerArgs::from_list(&args) {
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
                span: field.ty.span(),
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

    let ble = quote!(::nrf_softdevice::ble);

    for ch in &chars {
        let name_pascal = inflector::cases::pascalcase::to_pascal_case(&ch.name);
        let char_name = format_ident!("{}", ch.name);
        let value_handle = format_ident!("{}_value_handle", ch.name);
        let cccd_handle = format_ident!("{}_cccd_handle", ch.name);
        let get_fn = format_ident!("{}_get", ch.name);
        let set_fn = format_ident!("{}_set", ch.name);
        let notify_fn = format_ident!("{}_notify", ch.name);
        let indicate_fn = format_ident!("{}_indicate", ch.name);

        let uuid = ch.args.uuid;
        let read = ch.args.read;
        let write = ch.args.write;
        let notify = ch.args.notify;
        let indicate = ch.args.indicate;
        let ty = &ch.ty;
        let ty_as_val = quote!(<#ty as #ble::GattValue>);

        fields.push(syn::Field {
            ident: Some(value_handle.clone()),
            ty: syn::Type::Verbatim(quote!(u16).into()),
            attrs: Vec::new(),
            colon_token: Default::default(),
            vis: syn::Visibility::Inherited,
        });

        code_register_chars.extend(quote_spanned!(ch.span=>
            let #char_name = register_char(
                #ble::gatt_server::Characteristic {
                    uuid: #uuid,
                    can_read: #read,
                    can_write: #write,
                    can_notify: #notify,
                    can_indicate: #indicate,
                    max_len: #ty_as_val::MAX_SIZE as _,
                    vlen: #ty_as_val::MAX_SIZE != #ty_as_val::MIN_SIZE,
                },
                &[123],
            )?;
        ));

        code_register_init.extend(quote_spanned!(ch.span=>
            #value_handle: #char_name.value_handle,
        ));

        code_impl.extend(quote_spanned!(ch.span=>
            fn #get_fn(&self) -> Result<#ty, #ble::gatt_server::GetValueError> {
                let sd = unsafe { ::nrf_softdevice::Softdevice::steal() };
                let buf = &mut [0u8; #ty_as_val::MAX_SIZE];
                let size = #ble::gatt_server::get_value(sd, self.#value_handle, buf)?;
                Ok(#ty_as_val::from_gatt(&buf[..size]))
            }

            fn #set_fn(&self, val: #ty) -> Result<(), #ble::gatt_server::SetValueError> {
                let sd = unsafe { ::nrf_softdevice::Softdevice::steal() };
                let buf = #ty_as_val::to_gatt(&val);
                #ble::gatt_server::set_value(sd, self.#value_handle, buf)
            }
        ));

        if indicate || notify {
            fields.push(syn::Field {
                ident: Some(cccd_handle.clone()),
                ty: syn::Type::Verbatim(quote!(u16).into()),
                attrs: Vec::new(),
                colon_token: Default::default(),
                vis: syn::Visibility::Inherited,
            });
            code_register_init.extend(quote_spanned!(ch.span=>
                #cccd_handle: #char_name.cccd_handle,
            ));
        }

        if write {
            let case_write = format_ident!("{}Write", name_pascal);
            code_event_enum.extend(quote_spanned!(ch.span=>
                #case_write(#ty),
            ));
            code_on_write.extend(quote_spanned!(ch.span=>
                if handle == self.#value_handle {
                    return Some(#event_enum_name::#case_write(#ty_as_val::from_gatt(&data)));
                }
            ));
        }
        if notify {
            let case_enabled = format_ident!("{}NotificationsEnabled", name_pascal);
            let case_disabled = format_ident!("{}NotificationsDisabled", name_pascal);

            code_impl.extend(quote_spanned!(ch.span=>
                fn #notify_fn(
                    &self,
                    conn: &#ble::Connection,
                    val: #ty,
                ) -> Result<(), #ble::gatt_server::NotifyValueError> {
                    let buf = #ty_as_val::to_gatt(&val);
                    #ble::gatt_server::notify_value(conn, self.#value_handle, buf)
                }
            ));

            code_event_enum.extend(quote_spanned!(ch.span=>
                #case_enabled,
                #case_disabled,
            ));
            code_on_write.extend(quote_spanned!(ch.span=>
                if handle == self.#cccd_handle {
                    if data.len() != 0 && data[0] & 0x01 != 0 {
                        return Some(#event_enum_name::#case_enabled);
                    } else {
                        return Some(#event_enum_name::#case_disabled);
                    }
                }
            ));
        }
        if indicate {
            let case_enabled = format_ident!("{}IndicationsEnabled", name_pascal);
            let case_disabled = format_ident!("{}IndicationsDisabled", name_pascal);

            code_impl.extend(quote_spanned!(ch.span=>
                fn #indicate_fn(
                    &self,
                    conn: &#ble::Connection,
                    val: #ty,
                ) -> Result<(), #ble::gatt_server::IndicateValueError> {
                    let buf = #ty_as_val::to_gatt(&val);
                    #ble::gatt_server::indicate_value(conn, self.#value_handle, buf)
                }
            ));

            code_event_enum.extend(quote_spanned!(ch.span=>
                #case_enabled,
                #case_disabled,
            ));
            code_on_write.extend(quote_spanned!(ch.span=>
                if handle == self.#cccd_handle {
                    if data.len() != 0 && data[0] & 0x02 != 0 {
                        return Some(#event_enum_name::#case_enabled);
                    } else {
                        return Some(#event_enum_name::#case_disabled);
                    }
                }
            ));
        }
    }

    let uuid = args.uuid;
    struct_fields.named = syn::punctuated::Punctuated::from_iter(fields);

    let result = quote! {
        #struc

        impl #struct_name {
            #code_impl
        }

        impl #ble::gatt_server::Server for #struct_name {
            type Event = #event_enum_name;

            fn uuid() -> #ble::Uuid {
                #uuid
            }

            fn register<F>(service_handle: u16, mut register_char: F) -> Result<Self, #ble::gatt_server::RegisterError>
            where
                F: FnMut(#ble::gatt_server::Characteristic, &[u8]) -> Result<#ble::gatt_server::CharacteristicHandles, #ble::gatt_server::RegisterError>,
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

#[proc_macro_attribute]
pub fn gatt_client(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let mut struc = syn::parse_macro_input!(item as syn::ItemStruct);

    let args = match ServerArgs::from_list(&args) {
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
                .error("gatt_client structs must have named fields, not tuples.")
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
                span: field.ty.span(),
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
    let mut code_disc_new = TokenStream2::new();
    let mut code_disc_char = TokenStream2::new();
    let mut code_disc_done = TokenStream2::new();
    let mut code_event_enum = TokenStream2::new();

    let ble = quote!(::nrf_softdevice::ble);

    fields.push(syn::Field {
        ident: Some(format_ident!("conn")),
        ty: syn::Type::Verbatim(quote!(#ble::Connection).into()),
        attrs: Vec::new(),
        colon_token: Default::default(),
        vis: syn::Visibility::Inherited,
    });

    for ch in &chars {
        let name_pascal = inflector::cases::pascalcase::to_pascal_case(&ch.name);
        let uuid_field = format_ident!("{}_uuid", ch.name);
        let value_handle = format_ident!("{}_value_handle", ch.name);
        let cccd_handle = format_ident!("{}_cccd_handle", ch.name);
        let read_fn = format_ident!("{}_read", ch.name);
        let write_fn = format_ident!("{}_write", ch.name);
        let write_wor_fn = format_ident!("{}_write_without_response", ch.name);
        let write_try_wor_fn = format_ident!("{}_try_write_without_response", ch.name);

        let uuid = ch.args.uuid;
        let read = ch.args.read;
        let write = ch.args.write;
        let notify = ch.args.notify;
        let indicate = ch.args.indicate;
        let ty = &ch.ty;
        let ty_as_val = quote!(<#ty as #ble::GattValue>);

        fields.push(syn::Field {
            ident: Some(value_handle.clone()),
            ty: syn::Type::Verbatim(quote!(u16).into()),
            attrs: Vec::new(),
            colon_token: Default::default(),
            vis: syn::Visibility::Inherited,
        });

        fields.push(syn::Field {
            ident: Some(uuid_field.clone()),
            ty: syn::Type::Verbatim(quote!(#ble::Uuid).into()),
            attrs: Vec::new(),
            colon_token: Default::default(),
            vis: syn::Visibility::Inherited,
        });

        code_disc_new.extend(quote_spanned!(ch.span=>
            #value_handle: 0,
            #uuid_field: #uuid,
        ));

        let mut code_descs = TokenStream2::new();
        if indicate || notify {
            code_descs.extend(quote_spanned!(ch.span=>
                if _desc_uuid == #ble::Uuid::new_16(::nrf_softdevice::raw::BLE_UUID_DESCRIPTOR_CLIENT_CHAR_CONFIG as u16) {
                    self.#cccd_handle = desc.handle;
                }
            ));
        }

        code_disc_char.extend(quote_spanned!(ch.span=>
            if let Some(char_uuid) = characteristic.uuid {
                if char_uuid == self.#uuid_field {
                    // TODO maybe check the char_props have the necessary operations allowed? read/write/notify/etc
                    self.#value_handle = characteristic.handle_value;
                    for desc in descriptors {
                        if let Some(_desc_uuid) = desc.uuid {
                            #code_descs
                        }
                    }
                }
            }
        ));

        code_disc_done.extend(quote_spanned!(ch.span=>
            if self.#value_handle == 0 {
                return Err(#ble::gatt_client::DiscoverError::ServiceIncomplete);
            }
        ));

        if read {
            code_impl.extend(quote_spanned!(ch.span=>
                async fn #read_fn(&self) -> Result<#ty, #ble::gatt_client::ReadError> {
                    let mut buf = [0; #ty_as_val::MAX_SIZE];
                    let len = #ble::gatt_client::read(&self.conn, self.#value_handle, &mut buf).await?;
                    Ok(#ty_as_val::from_gatt(&buf[..len]))
                }
            ));
        }

        if write {
            code_impl.extend(quote_spanned!(ch.span=>
                async fn #write_fn(&self, val: #ty) -> Result<(), #ble::gatt_client::WriteError> {
                    let buf = #ty_as_val::to_gatt(&val);
                    #ble::gatt_client::write(&self.conn, self.#value_handle, buf).await
                }
                async fn #write_wor_fn(&self, val: #ty) -> Result<(), #ble::gatt_client::WriteError> {
                    let buf = #ty_as_val::to_gatt(&val);
                    #ble::gatt_client::write_without_response(&self.conn, self.#value_handle, buf).await
                }
                fn #write_try_wor_fn(&self, val: #ty) -> Result<(), #ble::gatt_client::TryWriteError> {
                    let buf = #ty_as_val::to_gatt(&val);
                    #ble::gatt_client::try_write_without_response(&self.conn, self.#value_handle, buf)
                }
            ));
        }

        if indicate || notify {
            fields.push(syn::Field {
                ident: Some(cccd_handle.clone()),
                ty: syn::Type::Verbatim(quote!(u16).into()),
                attrs: Vec::new(),
                colon_token: Default::default(),
                vis: syn::Visibility::Inherited,
            });
            code_disc_new.extend(quote_spanned!(ch.span=>
                #cccd_handle: 0,
            ));
            code_disc_done.extend(quote_spanned!(ch.span=>
                if self.#value_handle == 0 {
                    return Err(#ble::gatt_client::DiscoverError::ServiceIncomplete);
                }
            ));
        }

        if notify {
            let case_notification = format_ident!("{}Notification", name_pascal);
            code_event_enum.extend(quote_spanned!(ch.span=>
                #case_notification(#ty),
            ));
        }
    }

    let uuid = args.uuid;
    struct_fields.named = syn::punctuated::Punctuated::from_iter(fields);

    let result = quote! {
        #struc

        impl #struct_name {
            #code_impl
        }

        impl #ble::gatt_client::Client for #struct_name {
            //type Event = #event_enum_name;

            fn uuid() -> #ble::Uuid {
                #uuid
            }

            fn new_undiscovered(conn: #ble::Connection) -> Self {
                Self {
                    conn,
                    #code_disc_new
                }
            }

            fn discovered_characteristic(
                &mut self,
                characteristic: &#ble::gatt_client::Characteristic,
                descriptors: &[#ble::gatt_client::Descriptor],
            ) {
                #code_disc_char
            }

            fn discovery_complete(&mut self) -> Result<(), #ble::gatt_client::DiscoverError> {
                #code_disc_done
                Ok(())
            }
        }

        enum #event_enum_name {
            #code_event_enum
        }
    };
    result.into()
}
