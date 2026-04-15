use ewwii_plugin_api::{auto_plugin, PluginInfo};

auto_plugin!(
    VeryImpTest, 
    PluginInfo::new("com.very.imp", "0.1.0"),
    host, 
    {
        host.log("Hey there! I am sending this from a plugin!");
    }
);