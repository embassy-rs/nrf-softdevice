use darling::{Error, FromMeta};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SecurityMode {
    NoAccess,
    Open,
    JustWorks,
    Mitm,
    LescMitm,
    Signed,
    SignedMitm,
}

impl FromMeta for SecurityMode {
    fn from_string(value: &str) -> darling::Result<Self> {
        match value.trim().to_lowercase().as_str() {
            "noaccess" => Ok(SecurityMode::NoAccess),
            "open" => Ok(SecurityMode::Open),
            "justworks" => Ok(SecurityMode::JustWorks),
            "mitm" => Ok(SecurityMode::Mitm),
            "lescmitm" => Ok(SecurityMode::LescMitm),
            "signed" => Ok(SecurityMode::Signed),
            "signedmitm" => Ok(SecurityMode::SignedMitm),
            _ => Err(Error::unknown_value(format!(
                "SecurityMode {} is invalid. Expected one of: NoAccess, Open, JustWorks, Mitm, LescMitm, Signed, SignedMitm",
                value)
                .as_str())),
        }
    }
}

impl ToTokens for SecurityMode {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let literal = match self {
            SecurityMode::NoAccess => quote!(NoAccess),
            SecurityMode::Open => quote!(Open),
            SecurityMode::JustWorks => quote!(JustWorks),
            SecurityMode::Mitm => quote!(Mitm),
            SecurityMode::LescMitm => quote!(LescMitm),
            SecurityMode::Signed => quote!(Signed),
            SecurityMode::SignedMitm => quote!(SignedMitm),
        };
        tokens.extend(quote!(::nrf_softdevice::ble::SecurityMode::#literal))
    }
}
