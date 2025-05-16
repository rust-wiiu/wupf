#![no_std]
#![no_main]

use wupf::{hook_plugin, Handler, Plugin, StaticHandler};

use wut::{logger, println};

struct MyApp {
    a: u32,
}

impl StaticHandler for MyApp {
    fn handler() -> &'static Handler<Self> {
        static HANDLER: Handler<MyApp> = Handler::new();
        &HANDLER
    }
}

hook_plugin!(MyApp);
impl Plugin for MyApp {
    fn on_init() -> Self {
        let _ = logger::udp();

        println!("init");

        Self { a: 0 }
    }

    fn on_deinit(&mut self) {
        println!("deinit");

        logger::deinit();
    }

    fn on_start(&mut self) {
        println!("start");
    }

    fn on_exit(&mut self) {
        println!("exit");
    }
}
