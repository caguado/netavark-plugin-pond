use netavark::plugin::PluginExec;
use netavark_plugin_pond::NetNsDriver;

pub fn main() {
    let plugin: PluginExec<NetNsDriver> = NetNsDriver::default().into();
    plugin.exec();
}
