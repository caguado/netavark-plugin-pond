# netavark-plugin-pond

[![Crates.io](https://img.shields.io/crates/v/netavark-plugin-pond.svg)](https://crates.io/crates/netavark-plugin-pond)
[![Docs.rs](https://docs.rs/netavark-plugin-pond/badge.svg)](https://docs.rs/netavark-plugin-pond)
[![CI](https://github.com/caguado/netavark-plugin-pond/workflows/CI/badge.svg)](https://github.com/caguado/netavark-plugin-pond/actions)

## Motivation

Sometimes you want to deploy containerized services that are made up from multiple parts and are based on different container images. Sometimes those containers are deployed in a single-node hardware platform for which you don't want to run a kNs ecosystem. Sometimes that hardware platform is special enough that you want to implement the network datapath in user space and treat the kernel as the resource orchestrator for the underlay, the substrate of the virtual functions you aim to deliver.

This plugin is a proof of concept to deploy those types of container pods in a podman environment with a user-space datapath. The main goal is to build application components that are isolated from each other (e.g. sidecars using different container base images) _but_ share the network stack to communicate with one another and the outside. The usual threat model mitigation techniques that lead to enforce container network isolation are relaxed because the network datapath is also in user space and implemented elsewhere. There are tools like passt to implement tap/tun interfaces in such user space but the presence of OVS together with DPDK forces to implement some form of vhost, memif or other user space constructs.

This plugin is a playground to cover some of these cases and prototype gluing tools together. Currently, it creates veth pairs that still go through the kernel to connect containers to a OVS-DPDK dataplane. In the future, something else may bypass the kernel altogether.

Note, this is not an alternative to CNI, eBPF et al. by any means. It is an attempt to create an extension to the aforementioned use case for those environments where single-node podman, systemd, and some user space applications coexist.

### Example target

The first use case this plugin tackles is the handling of a kernel namespace unit as a service for sharing across the containers in a pod. This namespace is currently handled via scripting to produce the following unit file from a podman quadlet.

```ini
[Unit]
Description=${POD_DESCRIPTION} service network
After=openvswitch.service
Requires=openvswitch.service

[Network]
NetworkName=${POD_NETWORK}
Internal=true
Options=isolate=true
DisableDNS=true
Subnet=${POD_UNDERLAY_SUBNET}
PodmanArgs=--interface-name=${POD_NAME}null0

[Service]
# This section is ideally handled in the network segment above with this plugin
ExecStartPre=-ip netns add ${POD_NETWORK}
ExecStartPre=ip link add ${POD_LINK_UPSTREAM} type veth peer name ${POD_LINK_UPSTREAM}_${POD_LINK_NAME}
ExecStartPre=ip link set ${POD_LINK_UPSTREAM}_${POD_LINK_NAME} netns ${POD_NETWORK}
ExecStartPre=ip netns exec ${POD_NETWORK} ip link set dev ${POD_LINK_UPSTREAM}_${POD_LINK_NAME} name ${POD_LINK_NAME}
ExecStartPre=ip netns exec ${POD_NETWORK} ip link set ${POD_LINK_NAME} up
ExecStartPre=ip netns exec ${POD_NETWORK} ip addr add ${POD_SUBNET} dev ${POD_LINK_NAME}
ExecStartPre=ip netns exec ${POD_NETWORK} ip route add default via ${POD_SUBNET_GW} dev ${POD_LINK_NAME}
ExecStartPre=ip netns exec ${POD_NETWORK} ethtool --offload ${POD_LINK_NAME} tx off sg off tso off
ExecStartPre=ip netns exec ${POD_NETWORK} sysctl net.ipv4.ip_unprivileged_port_start=${POD_SERVICE_MIN_PORT}
ExecStartPre=ip link set ${POD_LINK_UPSTREAM} up
ExecStartPre=-ovs-vsctl add-port ${POD_BRIDGE_NAME} ${POD_LINK_UPSTREAM} tag=${POD_LINK_VLAN} vlan_mode=access -- set interface ${POD_LINK_UPSTREAM} external_ids:pod_id="${POD_NAME}" external_ids:pod_iface="${POD_LINK_UPSTREAM}"
ExecStartPre=ip netns exec ${POD_NETWORK} ip addr add 127.0.0.1/8 dev lo
ExecStartPre=ip netns exec ${POD_NETWORK} ip link set lo up

ExecStop=/usr/bin/podman network rm ${POD_NETWORK}
ExecStopPost=-ovs-vsctl del-port ${POD_BRIDGE_NAME} ${POD_LINK_UPSTREAM}
ExecStopPost=-ip netns exec ${POD_NETWORK} ip link set ${POD_LINK_NAME} netns 1
ExecStopPost=-ip link del ${POD_LINK_UPSTREAM}
ExecStopPost=-ip netns del ${POD_NETWORK}

[Install]
WantedBy=default.target
```

## Installation

### Cargo

* Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
* run `cargo install netavark-plugin-pond`

## License

Licensed under [Apache License](http://www.apache.org/licenses/LICENSE-2.0).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
