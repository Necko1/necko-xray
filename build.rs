const PROTO_FILES: &[&str] = &[
    // app/
    "proto/app/commander/config.proto",
    "proto/app/dispatcher/config.proto",
    "proto/app/dns/config.proto",
    "proto/app/dns/fakedns/fakedns.proto",
    "proto/app/log/config.proto",
    "proto/app/log/command/config.proto",
    "proto/app/metrics/config.proto",
    "proto/app/observatory/config.proto",
    "proto/app/observatory/burst/config.proto",
    "proto/app/observatory/command/command.proto",
    "proto/app/policy/config.proto",
    "proto/app/proxyman/config.proto",
    "proto/app/proxyman/command/command.proto",
    "proto/app/reverse/config.proto",
    "proto/app/router/config.proto",
    "proto/app/router/command/command.proto",
    "proto/app/stats/config.proto",
    "proto/app/stats/command/command.proto",
    "proto/app/version/config.proto",

    // common
    "proto/common/log/log.proto",
    "proto/common/net/address.proto",
    "proto/common/net/destination.proto",
    "proto/common/net/network.proto",
    "proto/common/net/port.proto",
    "proto/common/protocol/headers.proto",
    "proto/common/protocol/server_spec.proto",
    "proto/common/protocol/user.proto",
    "proto/common/serial/typed_message.proto",

    // core
    "proto/core/config.proto",

    // proxy
    "proto/proxy/blackhole/config.proto",
    "proto/proxy/dns/config.proto",
    "proto/proxy/dokodemo/config.proto",
    "proto/proxy/freedom/config.proto",
    "proto/proxy/http/config.proto",
    "proto/proxy/loopback/config.proto",
    "proto/proxy/shadowsocks/config.proto",
    "proto/proxy/shadowsocks_2022/config.proto",
    "proto/proxy/socks/config.proto",
    "proto/proxy/trojan/config.proto",
    "proto/proxy/vless/account.proto",
    "proto/proxy/vless/encoding/addons.proto",
    "proto/proxy/vless/inbound/config.proto",
    "proto/proxy/vless/outbound/config.proto",
    "proto/proxy/vmess/account.proto",
    "proto/proxy/vmess/inbound/config.proto",
    "proto/proxy/vmess/outbound/config.proto",
    "proto/proxy/wireguard/config.proto",

    // transport
    "proto/transport/internet/config.proto",
    "proto/transport/internet/grpc/config.proto",
    "proto/transport/internet/grpc/encoding/stream.proto",
    "proto/transport/internet/headers/dns/config.proto",
    "proto/transport/internet/headers/http/config.proto",
    "proto/transport/internet/headers/noop/config.proto",
    "proto/transport/internet/headers/srtp/config.proto",
    "proto/transport/internet/headers/tls/config.proto",
    "proto/transport/internet/headers/utp/config.proto",
    "proto/transport/internet/headers/wechat/config.proto",
    "proto/transport/internet/headers/wireguard/config.proto",
    "proto/transport/internet/httpupgrade/config.proto",
    "proto/transport/internet/kcp/config.proto",
    "proto/transport/internet/reality/config.proto",
    "proto/transport/internet/splithttp/config.proto",
    "proto/transport/internet/tcp/config.proto",
    "proto/transport/internet/tls/config.proto",
    "proto/transport/internet/udp/config.proto",
    "proto/transport/internet/websocket/config.proto",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(
            PROTO_FILES,
            &["proto"],
        )?;

    Ok(())
}