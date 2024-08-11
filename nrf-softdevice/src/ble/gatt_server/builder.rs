#![allow(dead_code)]

use core::marker::PhantomData;
use core::mem;
use core::ptr::null;

use super::characteristic::{self, AttributeMetadata, Presentation};
use super::{CharacteristicHandles, DescriptorHandle, IncludedServiceHandle, RegisterError, ServiceHandle};
use crate::ble::Uuid;
use crate::{raw, RawError, Softdevice};

pub struct ServiceBuilder<'a> {
    handle: u16,
    sd: PhantomData<&'a mut Softdevice>,
}

pub struct CharacteristicBuilder<'a> {
    handles: CharacteristicHandles,
    sb: PhantomData<&'a ServiceBuilder<'a>>,
}

impl<'a> ServiceBuilder<'a> {
    pub fn new(_sd: &'a mut Softdevice, uuid: Uuid) -> Result<Self, RegisterError> {
        let mut service_handle: u16 = 0;
        let ret = unsafe {
            raw::sd_ble_gatts_service_add(
                raw::BLE_GATTS_SRVC_TYPE_PRIMARY as u8,
                uuid.as_raw_ptr(),
                &mut service_handle as _,
            )
        };
        RawError::convert(ret)?;

        Ok(ServiceBuilder {
            handle: service_handle,
            sd: PhantomData,
        })
    }

    pub fn add_characteristic<T: AsRef<[u8]>>(
        &mut self,
        uuid: Uuid,
        attr: characteristic::Attribute<T>,
        md: characteristic::Metadata,
    ) -> Result<CharacteristicBuilder<'_>, RegisterError> {
        let value = attr.value.as_ref();
        let attr_md = attr.metadata.into_raw();
        self.add_characteristic_inner(uuid, value, attr.max_len, &attr_md, md)
    }

    fn add_characteristic_inner(
        &mut self,
        uuid: Uuid,
        value: &[u8],
        max_len: u16,
        attr_md: &raw::ble_gatts_attr_md_t,
        char_md: characteristic::Metadata,
    ) -> Result<CharacteristicBuilder<'_>, RegisterError> {
        assert!(value.len() <= usize::from(max_len));
        assert!(char_md
            .user_description
            .map_or(true, |x| x.value.len() <= usize::from(x.max_len)));

        let (char_props, char_ext_props) = char_md.properties.into_raw();
        let user_desc_md = char_md
            .user_description
            .and_then(|x| x.metadata.map(AttributeMetadata::into_raw));
        let cpfd_md = char_md.cpfd.map(Presentation::into_raw);
        let cccd_md = char_md.cccd.map(AttributeMetadata::into_raw);
        let sccd_md = char_md.sccd.map(AttributeMetadata::into_raw);

        let mut char_md = raw::ble_gatts_char_md_t {
            char_props,
            char_ext_props,
            p_char_user_desc: char_md.user_description.map_or(null(), |x| x.value.as_ptr()),
            char_user_desc_max_size: char_md.user_description.map_or(0, |x| x.max_len),
            char_user_desc_size: char_md.user_description.map_or(0, |x| x.value.len() as u16),
            p_char_pf: cpfd_md.as_ref().map_or(null(), |x| x as _),
            p_user_desc_md: user_desc_md.as_ref().map_or(null(), |x| x as _),
            p_cccd_md: cccd_md.as_ref().map_or(null(), |x| x as _),
            p_sccd_md: sccd_md.as_ref().map_or(null(), |x| x as _),
        };

        let mut attr = raw::ble_gatts_attr_t {
            p_uuid: uuid.as_raw_ptr(),
            p_attr_md: attr_md as _,
            init_len: unwrap!(value.len().try_into()),
            init_offs: 0,
            max_len,
            p_value: value.as_ptr() as *mut _,
        };

        let mut handles: raw::ble_gatts_char_handles_t = unsafe { mem::zeroed() };

        let ret = unsafe {
            raw::sd_ble_gatts_characteristic_add(self.handle, &mut char_md as _, &mut attr as _, &mut handles as _)
        };
        RawError::convert(ret)?;

        let handles = CharacteristicHandles {
            value_handle: handles.value_handle,
            user_desc_handle: handles.user_desc_handle,
            cccd_handle: handles.cccd_handle,
            sccd_handle: handles.sccd_handle,
        };

        Ok(CharacteristicBuilder {
            handles,
            sb: PhantomData,
        })
    }

    pub fn include_service(&mut self, service: &ServiceHandle) -> Result<IncludedServiceHandle, RegisterError> {
        let mut handle = 0;
        let ret = unsafe { raw::sd_ble_gatts_include_add(self.handle, service.0, &mut handle as _) };
        RawError::convert(ret)?;

        Ok(IncludedServiceHandle(handle))
    }

    pub fn build(self) -> ServiceHandle {
        ServiceHandle(self.handle)
    }
}

impl<'a> CharacteristicBuilder<'a> {
    pub fn add_descriptor<T: AsRef<[u8]>>(
        &mut self,
        uuid: Uuid,
        attr: characteristic::Attribute<T>,
    ) -> Result<DescriptorHandle, RegisterError> {
        let value = attr.value.as_ref();
        let attr_md = attr.metadata.into_raw();
        self.add_descriptor_inner(uuid, value, attr.max_len, &attr_md)
    }

    fn add_descriptor_inner(
        &mut self,
        uuid: Uuid,
        value: &[u8],
        max_len: u16,
        attr_md: &raw::ble_gatts_attr_md_t,
    ) -> Result<DescriptorHandle, RegisterError> {
        let attr = raw::ble_gatts_attr_t {
            p_uuid: uuid.as_raw_ptr(),
            p_attr_md: attr_md as _,
            init_len: unwrap!(value.len().try_into()),
            init_offs: 0,
            max_len,
            p_value: value.as_ptr() as *mut _,
        };

        let mut handle = 0;
        let ret = unsafe { raw::sd_ble_gatts_descriptor_add(self.handles.value_handle, &attr as _, &mut handle as _) };
        RawError::convert(ret)?;

        Ok(DescriptorHandle(handle))
    }

    pub fn build(self) -> CharacteristicHandles {
        self.handles
    }
}
