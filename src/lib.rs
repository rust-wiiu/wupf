//! Wii U Plugin Framework
//!
//! A framework to more conveniently write WUPS plugins in Rust.
//!
//! # Example
//!
//! ```
//! use plugin_framework::{hook_on_input, hook_on_update, hook_plugin, Handler, OnInput, OnUpdate, Plugin, StaticHandler};
//!
//! struct MyPlugin {
//!     a: u32,
//! }
//!
//! impl StaticHandler for MyPlugin {
//!     fn handler() -> &'static Handler<Self> {
//!         static HANDLER: Handler<MyPlugin> = Handler::new();
//!         &HANDLER
//!     }
//! }
//!
//! hook_plugin!(MyPlugin);
//! impl Plugin for MyPlugin {
//!     fn on_init() -> Self {
//!         Self { a: 0 }
//!     }
//!
//!     fn on_deinit(&mut self) {}
//!
//!     fn on_start(&mut self) {
//!         let _ = logger::udp();
//!
//!         self.a += 1;
//!         println!("start: {}", self.a);
//!     }
//!
//!     fn on_exit(&mut self) {
//!         self.a += 1;
//!         println!("end: {}", self.a);
//!
//!         logger::deinit();
//!     }
//! }
//! ```

#![no_std]

#[cfg(feature = "derive")]
pub use macros::PluginHandler;

use wut::{
    self,
    gamepad::{self, GamepadError},
    sync::{Mutex, OnceLock},
};

/// Plugin Handler
///
/// Contains the state of the plugin to allow synced and mutable state.
pub struct Handler<P> {
    inner: OnceLock<Mutex<P>>,
}

impl<P> Handler<P> {
    /// Create new plugin handler.
    pub const fn new() -> Self {
        Self {
            inner: OnceLock::new(),
        }
    }

    fn set(&self, p: P) {
        let _ = self.inner.set(Mutex::new(p));
    }

    fn get(&self) -> &Mutex<P> {
        self.inner.get().expect("Handler not initialized")
    }

    // fn take(&self) -> Mutex<P> {
    //     let mut inner = self.inner.borrow_mut();
    //     let value = core::mem::replace(&mut *inner, MaybeUninit::uninit());
    //     unsafe { value.assume_init() }
    // }
}

pub trait StaticHandler: 'static + Sized {
    /// Get the handler of the plugin.
    ///
    /// The recommended method of implementing this method is with a scoped static variable. This is identical to a global static but with the benefit of not exposing it to the outside.
    ///
    /// # Recommended
    ///
    /// ```
    /// struct MyPlugin;
    ///
    /// impl StaticHandler for MyPlugin {
    ///     fn handler() -> &'static Handler<Self> {
    ///         static HANDLER: Handler<MyPlugin> = Handler::new();
    ///         &HANDLER
    ///     }
    /// }
    /// ```
    fn handler() -> &'static Handler<Self>;
}

/// WUPS Plugin
///
///
pub trait Plugin: StaticHandler {
    /// Called when plugin is initialized / loaded.
    ///
    /// # Returns
    ///
    /// Value is used as inital state of plugin.
    fn on_init() -> Self;

    /// Called when plugin is deinitialized / unloaded.
    fn on_deinit(&mut self);

    /// Called when an application is started.
    fn on_start(&mut self);

    /// Called when an appilcation is existed.
    fn on_exit(&mut self);

    /// FFI callback for [on_init][Plugin::on_init]
    ///
    /// **Do not overwrite** this method unless you need to and know what you are doing!
    extern "C" fn ffi_on_init() {
        Self::handler().set(Self::on_init())
    }

    /// FFI callback for [on_deinit][Plugin::on_deinit]
    ///
    /// **Do not overwrite** this method unless you need to and know what you are doing!
    extern "C" fn ffi_on_deinit() {
        let handler = Self::handler().get();
        let mut app = handler.lock().unwrap();
        app.on_deinit();
    }

    /// FFI callback for [on_start][Plugin::on_start]
    ///
    /// **Do not overwrite** this method unless you need to and know what you are doing!
    extern "C" fn ffi_on_start() {
        let handler = Self::handler().get();
        let mut app = handler.lock().unwrap();
        app.on_start();
    }

    /// FFI callback for [on_exit][Plugin::on_exit]
    ///
    /// **Do not overwrite** this method unless you need to and know what you are doing!
    extern "C" fn ffi_on_exit() {
        let handler = Self::handler().get();
        let mut app = handler.lock().unwrap();
        app.on_exit();
    }
}

pub trait OnInput: Plugin {
    /// Called when the system reads the controller input.
    ///
    /// # Note
    ///
    /// Can be called multiple times per frame.
    ///
    /// # Returns
    ///
    /// Used to overwrite the controller input for the system.
    fn on_input(&mut self, port: gamepad::Port, state: gamepad::State) -> gamepad::State;

    /// FFI callback for [on_input][OnInput::on_input]
    ///
    /// **Do not overwrite** this method unless you need to and know what you are doing!
    extern "C" fn ffi_on_vpad(
        _chan: wut::bindings::VPADChan::Type,
        buffers: *mut wut::bindings::VPADStatus,
        _count: u32,
        error: *mut wut::bindings::VPADReadError::Type,
    ) /*-> i32*/
    {
        if GamepadError::try_from(unsafe { *error }).is_err() {
            return;
        }

        let handler = Self::handler().get();
        let mut app = handler.lock().unwrap();

        unsafe {
            *buffers &= app.on_input(gamepad::Port::DRC, gamepad::State::from(*buffers));
        }
    }

    /// FFI callback for [on_input][OnInput::on_input]
    ///
    /// **Do not overwrite** this method unless you need to and know what you are doing!
    extern "C" fn ffi_on_kpad(
        chan: wut::bindings::WPADChan::Type,
        data: *mut wut::bindings::KPADStatus,
        _size: u32,
        error: *mut wut::bindings::KPADError::Type,
    ) /* -> i32 */
    {
        if GamepadError::try_from(unsafe { *error }).is_err() {
            return;
        }

        let handler = Self::handler().get();
        let mut app = handler.lock().unwrap();

        unsafe {
            *data &= app.on_input(gamepad::Port::from_wpad(chan), gamepad::State::from(*data));
        }
    }
}

pub trait OnUpdate: Plugin {
    /// Called after the system updates the screen / every frame.
    fn on_update(&mut self);

    /// FFI callback for [on_update][OnUpdate::on_update]
    ///
    /// **Do not overwrite** this method unless you need to and know what you are doing!
    extern "C" fn ffi_on_update() {
        let handler = Self::handler().get();
        let mut app = handler.lock().unwrap();
        app.on_update();
    }
}

#[macro_export]
macro_rules! hook_plugin {
    ($plugin:ident) => {
        ::wups::wups_hook_ex!("INIT_PLUGIN", $plugin::ffi_on_init);
        ::wups::wups_hook_ex!("DEINIT_PLUGIN", $plugin::ffi_on_deinit);
        ::wups::wups_hook_ex!("APPLICATION_STARTS", $plugin::ffi_on_start);
        ::wups::wups_hook_ex!("APPLICATION_REQUESTS_EXIT", $plugin::ffi_on_exit);
    };
}

#[macro_export]
macro_rules! hook_on_input {
    ($plugin:ident) => {
        #[::wups::function_hook(module = VPAD, function = VPADRead)]
        fn plugin_VPADRead(
            chan: ::wut::bindings::VPADChan::Type,
            buffers: *mut ::wut::bindings::VPADStatus,
            count: u32,
            error: *mut ::wut::bindings::VPADReadError::Type,
        ) -> i32 {
            let status = unsafe { hooked(chan, buffers, count, error) };

            $plugin::ffi_on_vpad(chan, buffers, count, error);

            status
        }

        #[::wups::function_hook(module = PADSCORE, function = KPADReadEx)]
        fn plugin_KPADReadEx(
            chan: ::wut::bindings::WPADChan::Type,
            data: *mut ::wut::bindings::KPADStatus,
            size: u32,
            error: *mut ::wut::bindings::KPADError::Type,
        ) -> i32 {
            let status = unsafe { hooked(chan, data, size, error) };

            $plugin::ffi_on_kpad(chan, data, size, error);

            status
        }
    };
}

#[macro_export]
macro_rules! hook_on_update {
    ($plugin:ident) => {
        #[::wups::function_hook(module = GX2, function = GX2SwapScanBuffers)]
        fn plugin_GX2SwapScanBuffers() {
            unsafe {
                hooked();
            }

            $plugin::ffi_on_update();
        }
    };
}
