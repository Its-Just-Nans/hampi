//! `APER` Code generation for ASN.1 Choice Type

use proc_macro::TokenStream;
use quote::quote;

use crate::attrs::{parse_fld_meta_as_codec_params, TyCodecParams};

pub(super) fn generate_aper_decode_for_asn_choice(
    ast: &syn::DeriveInput,
    params: &TyCodecParams,
) -> proc_macro::TokenStream {
    let name = &ast.ident;

    let lb = params.lb.as_ref().unwrap().value().parse::<i128>().unwrap();
    let ub = params.ub.as_ref().unwrap().value().parse::<i128>().unwrap();
    let ext = params.ext.as_ref();

    let variant_tokens = generate_choice_variant_tokens_using_attrs(ast);
    if variant_tokens.is_err() {
        return variant_tokens.err().unwrap().to_compile_error().into();
    }
    let variant_tokens = variant_tokens.unwrap();

    let tokens = quote! {

        impl asn1_codecs::aper::AperCodec for #name {
            type Output = Self;

            fn decode(data: &mut asn1_codecs::aper::AperCodecData) -> Result<Self::Output, asn1_codecs::aper::AperCodecError> {

                let (idx, extended) = asn1_codecs::aper::decode::decode_choice_idx(data, #lb, #ub, #ext)?;
                if !extended {
                    match idx {
                        #(#variant_tokens)*
                        _ => Err(asn1_codecs::aper::AperCodecError::new(format!("Index {} is not a valid Choice Index", idx).as_str()))
                    }
                } else {
                    Err(asn1_codecs::aper::AperCodecError::new("CHOICE Additions not supported yet."))
                }
            }
        }
    };

    TokenStream::from(tokens)
}

fn generate_choice_variant_tokens_using_attrs(
    ast: &syn::DeriveInput,
) -> Result<Vec<proc_macro2::TokenStream>, syn::Error> {
    let mut tokens = vec![];

    let mut errors = vec![];
    if let syn::Data::Enum(ref data) = ast.data {
        for variant in &data.variants {
            let codec_params = parse_fld_meta_as_codec_params(&variant.attrs);
            match codec_params {
                Err(e) => errors.push(e),
                Ok(cp) => {
                    let key = cp.key.as_ref();
                    if key.is_none() {
                        errors.push(syn::Error::new_spanned(
                        variant,
                        "Missing Key for the variant. Please provide `#[asn(key = <int>)]` attribute.",
                    ));
                        continue;
                    }
                    let _extended = cp.extended.as_ref();
                    let variant_ident = &variant.ident;
                    if let syn::Fields::Unnamed(ref fields) = variant.fields {
                        if fields.unnamed.len() == 1 {
                            let ty = &fields.unnamed.first().as_ref().unwrap().ty;
                            let variant_token = quote! {
                                #key => Ok(Self::#variant_ident(#ty::decode(data)?)),
                            };
                            tokens.push(variant_token);
                        } else {
                            errors.push(syn::Error::new_spanned(
                                variant,
                                "Unsupported variant type".to_string(),
                            ));
                        }
                    }
                }
            }
        }
    }

    if let Some((first, others)) = errors.split_first_mut() {
        for e in others {
            first.combine(e.clone())
        }
        Err(first.clone())
    } else {
        Ok(tokens)
    }
}
