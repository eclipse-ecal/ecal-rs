/********************************************************************************
 * Copyright (c) 2024 Kopernikus Automotive
 * 
 * This program and the accompanying materials are made available under the
 * terms of the Apache License, Version 2.0 which is available at
 * https://www.apache.org/licenses/LICENSE-2.0.
 * 
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
 * License for the specific language governing permissions and limitations
 * under the License.
 * 
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

use anyhow::Result;
use std::{
    env, ffi,
    marker::PhantomData,
    os::raw::{c_char, c_int, c_long, c_longlong, c_void},
    ptr, slice,
    time::{Duration, Instant},
};
use thiserror::Error;

#[cfg(feature = "derive")]
pub use ecal_derive::Message;

pub mod sys;

pub trait Message {
    fn type_name() -> &'static str;
}

// TODO: ... yeah
#[derive(Debug, Error)]
pub enum CalError {
    #[error("eCAL was not initialized.")]
    InitializationFailed,
    #[error("Unable to create new publisher for `{0}`")]
    PublisherCreationFailed(String),
    #[error("Unable to create new subscriber for `{0}`")]
    SubscriberCreationFailed(String),
    #[error("The message was not sent or was only partially sent.")]
    PublishFailed,
    #[error("Unexpected message format")]
    InvalidFormat,
    #[error("Time-out waiting to receive message.")]
    Timeout,
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

pub mod format {
    use anyhow::Result;

    pub trait Format {
        fn topic_type() -> String;
        fn topic_description() -> Option<String>;
    }

    pub trait Serializer<T> {
        fn serialize(message: &T, buffer: &mut Vec<u8>) -> Result<()>;
    }

    pub trait Deserializer<'a, T> {
        fn deserialize(buffer: &'a [u8]) -> Result<T>;
    }

    #[cfg(feature = "use_msgpack")]
    pub mod msgpack {
        use super::{Deserializer, Format, Serializer};
        use anyhow::{Error, Result};
        use serde::{Deserialize, Serialize};
        use std::marker::PhantomData;

        pub struct MessagePack<T: crate::Message> {
            _ty: PhantomData<T>,
        }

        impl<T> Format for MessagePack<T>
        where
            T: crate::Message,
        {
            fn topic_type() -> String {
                format!("mpack:{}", T::type_name())
            }

            /// unsupported by msgpack serialization
            fn topic_description() -> Option<String> {
                None
            }
        }

        impl<T> Serializer<T> for MessagePack<T>
        where
            T: Serialize + crate::Message,
        {
            fn serialize(message: &T, buf: &mut Vec<u8>) -> Result<()> {
                rmp_serde::encode::write(buf, message).map_err(Error::from)
            }
        }

        impl<'a, T> Deserializer<'a, T> for MessagePack<T>
        where
            T: Deserialize<'a> + crate::Message,
        {
            fn deserialize(buffer: &'a [u8]) -> Result<T> {
                rmp_serde::from_slice(buffer).map_err(Error::from)
            }
        }
    }

    #[cfg(feature = "use_prost")]
    pub mod prost {
        use super::{Deserializer, Format, Serializer};
        pub use ::prost::Message as ProstMessage;
        use anyhow::{Error, Result};
        use std::marker::PhantomData;

        pub struct Prost<T: crate::Message + ::prost::Message> {
            _ty: PhantomData<T>,
        }

        impl<T> Format for Prost<T>
        where
            T: crate::Message + ::prost::Message,
        {
            fn topic_type() -> String {
                format!("proto:{}", T::type_name())
            }

            /// unsupported by prost.
            fn topic_description() -> Option<String> {
                None
            }
        }

        impl<T> Serializer<T> for Prost<T>
        where
            T: crate::Message + ::prost::Message,
        {
            fn serialize(message: &T, buf: &mut Vec<u8>) -> Result<()> {
                message.encode(buf).map_err(Error::from)
            }
        }

        impl<'a, T> Deserializer<'a, T> for Prost<T>
        where
            T: crate::Message + ::prost::Message + Default,
        {
            fn deserialize(buffer: &'a [u8]) -> Result<T> {
                T::decode(buffer).map_err(Error::from)
            }
        }
    }

    #[cfg(feature = "use_protobuf")]
    pub mod protobuf {
        use super::{Deserializer, Format, Serializer};
        use anyhow::{Error, Result};
        use std::marker::PhantomData;

        pub struct Protobuf<T: ::protobuf::Message> {
            _ty: PhantomData<T>,
        }

        impl<T> Format for Protobuf<T>
        where
            T: crate::Message + ::protobuf::Message,
        {
            fn topic_type() -> String {
                format!("proto:{}", T::type_name())
            }

            fn topic_description() -> Option<String> {
                log::warn!("Topic descriptions do not yet work.");
                let descriptor = T::descriptor_static();
                let _pset = ::protobuf::descriptor::FileDescriptorSet::default();
                let description = ::protobuf::text_format::print_to_string(descriptor.get_proto());
                Some(description)
            }
        }

        impl<T> Serializer<T> for Protobuf<T>
        where
            T: crate::Message + ::protobuf::Message,
        {
            fn serialize(message: &T, buf: &mut Vec<u8>) -> Result<()> {
                message.write_to_vec(buf).map_err(Error::from)
            }
        }

        impl<'a, T> Deserializer<'a, T> for Protobuf<T>
        where
            T: ::protobuf::Message + Default,
        {
            fn deserialize(buffer: &'a [u8]) -> Result<T> {
                T::parse_from_bytes(buffer).map_err(Error::from)
            }
        }
    }

    #[cfg(feature = "use_capnp")]
    pub mod capnp {
        use anyhow::Result;
        use std::marker::PhantomData;

        use super::{Deserializer, Format, Serializer};
        use capnp::{
            message::{ReaderOptions, TypedBuilder, TypedReader},
            serialize::{read_message_from_flat_slice, write_message_to_words, SliceSegments},
            traits::Owned,
        };

        pub struct Capnp<T>
        where
            T: crate::Message + Owned,
        {
            _t: PhantomData<T>,
        }

        impl<T> Format for Capnp<T>
        where
            T: crate::Message + Owned,
        {
            fn topic_type() -> String {
                format!("capnp:{}", T::type_name())
            }

            fn topic_description() -> Option<String> {
                None
            }
        }

        impl<T> Serializer<TypedBuilder<T>> for Capnp<T>
        where
            T: crate::Message + Owned,
        {
            fn serialize(message: &TypedBuilder<T>, buffer: &mut Vec<u8>) -> Result<()> {
                buffer.append(&mut write_message_to_words(message.borrow_inner()));
                Ok(())
            }
        }

        impl<'a, T> Deserializer<'a, TypedReader<SliceSegments<'a>, T>> for Capnp<T>
        where
            T: crate::Message + Owned,
        {
            fn deserialize(mut buffer: &'a [u8]) -> Result<TypedReader<SliceSegments<'a>, T>> {
                Ok(read_message_from_flat_slice(&mut buffer, ReaderOptions::default())?.into())
            }
        }
    }
}

#[cfg(feature = "use_msgpack")]
pub mod msgpack {
    use super::format::msgpack::MessagePack;
    pub type Publisher<T> = super::Publisher<T, MessagePack<T>>;
    pub type Subscriber<T> = super::Subscriber<T, MessagePack<T>>;
}

#[cfg(feature = "use_prost")]
pub mod prost {
    use super::format::prost::Prost;
    pub type Publisher<T> = super::Publisher<T, Prost<T>>;
    pub type Subscriber<T> = super::Subscriber<T, Prost<T>>;
}

#[cfg(feature = "use_protobuf")]
pub mod protobuf {
    use super::format::protobuf::Protobuf;
    pub type Publisher<T> = super::Publisher<T, Protobuf<T>>;
    pub type Subscriber<T> = super::Subscriber<T, Protobuf<T>>;
}

#[cfg(feature = "use_capnp")]
pub mod capnp {
    use capnp::{
        message::{TypedBuilder, TypedReader},
        serialize::SliceSegments,
    };

    use super::format::capnp::Capnp;
    pub type Publisher<T> = super::Publisher<TypedBuilder<T>, Capnp<T>>;
    pub type Subscriber<'a, T> = super::Subscriber<TypedReader<SliceSegments<'a>, T>, Capnp<T>>;
}

pub struct Publisher<T, S> {
    handle: sys::ECAL_HANDLE,
    _ty: PhantomData<T>,
    _serializer: PhantomData<S>,
}

impl<T, S> Publisher<T, S>
where
    S: format::Format + format::Serializer<T>,
{
    pub fn new(topic_name: &str) -> Result<Self> {
        let handle = unsafe { sys::eCAL_Pub_New() };
        let c_topic_name = ffi::CString::new(topic_name)?;
        let c_topic_type = ffi::CString::new(S::topic_type())?;
        let description = S::topic_description();
        let c_description = match description {
            Some(description) => ffi::CString::new(description)?,
            None => ffi::CString::default(),
        };
        let status = unsafe {
            sys::eCAL_Pub_Create(
                handle,
                c_topic_name.as_ptr(),
                c_topic_type.as_ptr(),
                c_description.as_ptr() as *const std::os::raw::c_char,
                c_description.as_bytes().len() as i32,
            )
        };
        if status == 0 {
            unsafe {
                sys::eCAL_Pub_Destroy(handle);
            }
            Err(CalError::PublisherCreationFailed(topic_name.to_string()).into())
        } else {
            Ok(Publisher {
                handle,
                _serializer: Default::default(),
                _ty: Default::default(),
            })
        }
    }

    pub fn set_id(&mut self, id: i64) -> bool {
        unsafe { sys::eCAL_Pub_SetID(self.handle, id as c_longlong) != 0 }
    }

    pub fn shm_set_buffer_count(&mut self, buffer_num: usize) -> bool {
        unsafe { sys::eCAL_Pub_ShmSetBufferCount(self.handle, buffer_num as c_long) != 0 }
    }

    pub fn is_subscribed(&self) -> bool {
        unsafe { sys::eCAL_Pub_IsSubscribed(self.handle) != 0 }
    }

    pub fn send(&self, msg: &T) -> Result<()> {
        self.send_with_time(msg, -1)
    }

    /// Same as [send](#method.send) but let the caller set the time of the message
    pub fn send_with_time(&self, msg: &T, time: i64) -> Result<()> {
        let mut buf = Vec::with_capacity(32);
        S::serialize(msg, &mut buf)?;

        let bytes_expected = buf.len();
        let bytes_sent = unsafe {
            sys::eCAL_Pub_Send(
                self.handle,
                buf.as_ptr() as *const c_void,
                bytes_expected as c_int,
                time as c_longlong,
            )
        };
        log::trace!("Published {} / {} bytes", bytes_sent, bytes_expected);
        if bytes_sent != bytes_expected as c_int {
            Err(CalError::PublishFailed.into())
        } else {
            Ok(())
        }
    }

    unsafe extern "C" fn event_wrapper<F>(
        _topic_name: *const c_char,
        _data: *const sys::SPubEventCallbackDataC,
        ctx: *mut c_void,
    ) where
        F: FnMut(),
    {
        // TODO: Support other callback types. :D
        let cb_ptr = ctx as *mut F;
        let callback = &mut *cb_ptr;
        callback();
    }

    pub fn on_subscribed<F>(&self, callback: F)
    where
        F: FnMut(),
    {
        // TODO: memory leak?
        let callback = Box::into_raw(Box::new(callback));
        unsafe {
            sys::eCAL_Pub_AddEventCallbackC(
                self.handle,
                sys::eCAL_Publisher_Event::pub_event_connected,
                Some(Self::event_wrapper::<F>),
                callback as *mut _,
            );
        }
    }
}

impl<T, S> Drop for Publisher<T, S> {
    fn drop(&mut self) {
        unsafe {
            sys::eCAL_Pub_Destroy(self.handle);
        }
    }
}

pub type RecvFn<T> = dyn Fn(Instant, T);

pub struct Subscriber<T, D> {
    handle: sys::ECAL_HANDLE,
    _ty: PhantomData<T>,
    _deserializer: PhantomData<D>,
}

impl<'a, T, D> Subscriber<T, D>
where
    D: format::Format + format::Deserializer<'a, T>,
{
    pub fn new(topic_name: &str) -> Result<Self> {
        let handle = unsafe { sys::eCAL_Sub_New() };
        let c_topic_name = ffi::CString::new(topic_name)?;
        let c_topic_type = ffi::CString::new(D::topic_type())?;
        let description = D::topic_description();
        let c_description = match description {
            Some(description) => ffi::CString::new(description)?,
            None => ffi::CString::default(),
        };
        let status = unsafe {
            sys::eCAL_Sub_Create(
                handle,
                c_topic_name.as_ptr(),
                c_topic_type.as_ptr(),
                c_description.as_ptr() as *const std::os::raw::c_char,
                c_description.as_bytes().len() as i32,
            )
        };
        if status == 0 {
            Err(CalError::SubscriberCreationFailed(topic_name.to_string()).into())
        } else {
            Ok(Subscriber {
                handle,
                _ty: Default::default(),
                _deserializer: Default::default(),
            })
        }
    }

    fn _recv(&self, timeout: c_int) -> Result<T> {
        let mut buf = ptr::null_mut::<c_void>();
        let buf_len = sys::ECAL_ALLOCATE_4ME as i32;
        let mut time = 0;

        let bytes_received =
            unsafe { sys::eCAL_Sub_Receive(self.handle, &mut buf, buf_len, &mut time, timeout) };

        if bytes_received > 0 {
            let bytes = unsafe { slice::from_raw_parts(buf as *const u8, bytes_received as usize) };

            let res = D::deserialize(bytes).map_err(|err| {
                log::error!("Failed to decode message: {}", err);
                CalError::InvalidFormat.into()
            });

            log::trace!("Freeing recv buffer");
            unsafe {
                sys::eCAL_FreeMem(buf);
            }

            res
        } else {
            log::trace!("Subscriber timeout");
            if !buf.is_null() {
                log::warn!("Non-null pointer returned from recv, but bytes_received was 0!");
            }
            Err(CalError::Timeout.into())
        }
    }

    pub fn recv(&self) -> Result<T> {
        log::trace!("Subscriber::recv");
        self._recv(-1).map_err(Into::into)
    }

    pub fn try_recv(&self, timeout: Duration) -> Option<T> {
        log::trace!("Subscriber::try_recv");
        let timeout = timeout.as_millis() as c_int;
        self._recv(timeout).ok()
    }

    unsafe extern "C" fn recv_wrapper<F>(
        _topic_name: *const c_char,
        data: *const sys::SReceiveCallbackDataC,
        ctx: *mut c_void,
    ) where
        F: FnMut(Instant, T),
    {
        let bytes = slice::from_raw_parts((*data).buf as *const u8, (*data).size as usize);

        if let Ok(msg) = D::deserialize(bytes) {
            log::trace!("Received {} bytes", bytes.len());
            let cb_ptr = ctx as *mut F;
            let callback = &mut *cb_ptr;
            // TODO: use eCAL timestamp
            let timestamp = Instant::now();
            callback(timestamp, msg);
        } else {
            log::error!("Failed to decode message.");
        }
    }

    pub fn on_recv<'b, F: FnMut(Instant, T) + 'b>(&'b self, callback: F) {
        // TODO: memory leak?
        let callback = Box::into_raw(Box::new(callback));
        unsafe {
            sys::eCAL_Sub_AddReceiveCallbackC(
                self.handle,
                Some(Self::recv_wrapper::<F>),
                callback as *mut _,
            );
        }
    }

    unsafe extern "C" fn recv_wrapper_full<F>(
        _topic_name: *const c_char,
        data: *const sys::SReceiveCallbackDataC,
        ctx: *mut c_void,
    ) where
        F: FnMut(sys::SReceiveCallbackDataC, T),
    {
        let bytes = slice::from_raw_parts((*data).buf as *const u8, (*data).size as usize);

        if let Ok(msg) = D::deserialize(bytes) {
            log::trace!("Received {} bytes", bytes.len());
            let cb_ptr = ctx as *mut F;
            let callback = &mut *cb_ptr;
            callback(*data, msg);
        } else {
            log::error!("Failed to decode message.");
        }
    }

    /// Same as [`on_recv`](#method.on_recv), but instead of pass the Instant of the message this will pass
    /// the entire content that arrives from the receive callback ([SReceiveCallbackDataC](sys::SReceiveCallbackDataC))
    pub fn on_recv_full<'b, F: FnMut(sys::SReceiveCallbackDataC, T) + 'b>(&'b self, callback: F) {
        // TODO: memory leak?
        let callback = Box::into_raw(Box::new(callback));
        unsafe {
            sys::eCAL_Sub_AddReceiveCallbackC(
                self.handle,
                Some(Self::recv_wrapper_full::<F>),
                callback as *mut _,
            );
        }
    }
}

impl<T, D> Drop for Subscriber<T, D> {
    fn drop(&mut self) {
        unsafe {
            sys::eCAL_Sub_Destroy(self.handle);
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NodeState {
    Healthy,
    Critical,
    Failed,
    Unknown,
    Warning,
}

impl From<NodeState> for sys::eCAL_Process_eSeverity {
    fn from(state: NodeState) -> Self {
        use sys::eCAL_Process_eSeverity::*;
        use NodeState::*;
        match state {
            Healthy => proc_sev_healthy,
            Critical => proc_sev_critical,
            Failed => proc_sev_failed,
            Unknown => proc_sev_unknown,
            Warning => proc_sev_warning,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SeverityLevel {
    Level1,
    Level2,
    Level3,
    Level4,
    Level5,
}

impl From<SeverityLevel> for sys::eCAL_Process_eSeverity_Level {
    fn from(level: SeverityLevel) -> Self {
        use sys::eCAL_Process_eSeverity_Level::*;
        use SeverityLevel::*;
        match level {
            Level1 => proc_sev_level1,
            Level2 => proc_sev_level2,
            Level3 => proc_sev_level3,
            Level4 => proc_sev_level4,
            Level5 => proc_sev_level5,
        }
    }
}

#[derive(Debug, Default)]
pub struct Cal {
    status_msg: ffi::CString,
}

impl Cal {
    pub fn new(unit_name: &str) -> Result<Self> {
        initialize(unit_name).and_then(|_| {
            let mut cal = Cal::default();
            cal.set_state(NodeState::Healthy, SeverityLevel::Level1, "ok")?;
            Ok(cal)
        })
    }

    /// Sets the process state and severity level.
    /// Can fail if the status message is unable to be
    /// converted to a CString.
    pub fn set_state(&mut self, state: NodeState, level: SeverityLevel, info: &str) -> Result<()> {
        self.status_msg = ffi::CString::new(info)?;
        unsafe { sys::eCAL_Process_SetState(state.into(), level.into(), self.status_msg.as_ptr()) };
        Ok(())
    }
}

impl Drop for Cal {
    fn drop(&mut self) {
        finalize();
    }
}

fn initialize(unit_name: &str) -> Result<()> {
    let mut args = env::args()
        .map(|arg| ffi::CString::new(arg).expect("Failed to build CString from arg"))
        .collect::<Vec<ffi::CString>>();
    let mut argv = args
        .iter_mut()
        .map(|a| a.as_ptr() as *mut c_char)
        .collect::<Vec<*mut c_char>>();
    let argc = argv.len() as c_int;

    let c_unit_name = ffi::CString::new(unit_name).expect("Failed to build CString from unit_name");

    let status = unsafe {
        sys::eCAL_Initialize(
            argc,
            argv.as_mut_ptr(),
            c_unit_name.as_ptr(),
            sys::eCAL_Init_Default,
        )
    };

    match status {
        -1 => {
            log::error!("Failed to initialize eCAL");
            return Err(CalError::InitializationFailed.into());
        }
        0 => log::info!("eCAL initiailized as '{}'.", unit_name),
        1 => log::warn!("eCAL was already initialized."),
        _ => log::warn!(
            "Unexpected status returned from eCAL_Initialize: {}",
            status
        ),
    }

    Ok(())
}

fn finalize() {
    unsafe {
        log::debug!("Finalizing eCAL system.");
        let _ = sys::eCAL_Finalize(sys::eCAL_Init_All);
    }
}

pub fn ok() -> bool {
    let status = unsafe { sys::eCAL_Ok() };
    log::trace!("eCAL_Ok == {}", status);
    status != 0
}

pub fn sleep(duration: Duration) {
    let ms = duration.as_millis();
    unsafe {
        sys::eCAL_Process_SleepMS(ms as c_long);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ecal_init_and_finalize() {
        let _ = ::env_logger::try_init();
        assert!(super::initialize("kcal_init_test").is_ok());
        super::sleep(std::time::Duration::from_millis(10));
        super::finalize();
    }

    #[test]
    fn ecal_init_and_finalize_raii() {
        let _ = ::env_logger::try_init();
        {
            let cal = super::Cal::new("kcal_tests");
            assert!(cal.is_ok());
            let _cal = cal.unwrap();
            assert!(super::ok());
        }
    }

    #[test]
    fn ecal_set_state() {
        let _ = ::env_logger::try_init();
        {
            let cal = super::Cal::new("kcal_tests");
            assert!(cal.is_ok());
            let mut cal = cal.unwrap();
            cal.set_state(
                super::NodeState::Healthy,
                super::SeverityLevel::Level1,
                "All good in the hood!",
            )
            .expect("Unable to set eCAL node state.");
            super::sleep(std::time::Duration::from_millis(20));
            assert!(super::ok());
        }
    }
}

unsafe impl<T, S> Send for Publisher<T, S> {}
unsafe impl<T, S> Sync for Publisher<T, S> {}
