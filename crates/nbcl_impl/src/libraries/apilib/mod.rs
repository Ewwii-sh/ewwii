mod linux;
// mod wifi;

use nbcl::{
    library::{Library, LibraryItem},
    NbclEngine, Type,
};

pub fn register_api_lib(engine: &mut NbclEngine) {
    let linux = LibraryItem::define("linux")
        .with_fn("get_kernel_version", vec![], Type::Str, linux::get_kernel_version)
        .with_fn("get_battery_perc", vec![], Type::Int, linux::get_battery_perc)
        .with_fn("get_cpu_info", vec![], Type::List, linux::get_cpu_info)
        .with_fn("get_ram_info", vec![], Type::Map, linux::get_ram_info)
        .with_fn("get_gpu_info", vec![], Type::List, linux::get_gpu_info)
        .with_fn("get_disk_info", vec![], Type::Map, linux::get_disk_info);

    // let wifi = LibraryItem::define("wifi")
    //     .with_fn("scan", vec![], Type::List, wifi::scan)
    //     .with_fn("current_connection", vec![], Type::Map, wifi::current_connection)
    //     .with_fn("connect", vec![Type::Str, Type::Str], Type::Null, wifi::connect)
    //     .with_fn("connect_without_password", vec![Type::Str], Type::Null, wifi::connect_without_password)
    //     .with_fn("disconnect", vec![], Type::Null, wifi::disconnect)
    //     .with_fn("disable_adapter", vec![], Type::Null, wifi::disable_adapter)
    //     .with_fn("enable_adapter", vec![], Type::Null, wifi::enable_adapter)
    //     .with_fn("get_adapter_connectivity", vec![], Type::Null, wifi::get_adapter_connectivity)

    let api_lib = Library::new("api".into(), vec![linux]);
    engine.register_library(api_lib);
}
