use std::env;
use json_value_merge::Merge;
use lazy_static::lazy_static;
use serde_json::{json, Value};

lazy_static!(
    static ref API: Value = json!({
        "api": {
            "tag": "api",
            "services": [
                "HandlerService",
                "StatsService",
                "LoggerService",
                "RoutingService",
                "ReflectionService"
            ]
        }
    });

    static ref STATS: Value = json!({
        "stats": {}
    });

    static ref POLICY: Value = json!({
        "policy": {
            "levels": {
                "0": {
                    "statsUserUplink": true,
                    "statsUserDownlink": true,
                    "statsUserOnline": true
                }
            },
            "system": {
                "statsInboundUplink": true,
                "statsInboundDownlink": true,
                "statsOutboundUplink": true,
                "statsOutboundDownlink": true
            }
        }
    });

    static ref INBOUNDS: Value = json!({
        "inbounds": [
            {
                "listen": "127.0.0.1",
                "port": env::var("XRAY_API_PORT")
                    .map_err(|_| anyhow::anyhow!("XRAY_API_PORT not found"))
                    .unwrap(),
                "protocol": "dokodemo-door",
                "settings": {
                    "address": "127.0.0.1"
                },
                "tag": "api"
            }
        ]
    });

    static ref ROUTING: Value = json!({
        "routing": {
            "rules": [
                {
                    "inboundTag": [
                        "api"
                    ],
                    "outboundTag": "api"
                }
            ]
        }
    });
);

pub fn get_config_from_profile(
    path: Option<&str>
) -> anyhow::Result<Value> {
    let mut profile = match path {
        Some(p) => {
            let mut profile = serde_json::from_str::<Value>(
                &std::fs::read_to_string(p)?
            )?;

            if !profile.is_object() {
                profile = json!({});
            }

            profile
        },
        None => json!({})
    };

    profile.merge(&API.clone());
    profile.merge(&STATS.clone());
    profile.merge(&POLICY.clone());
    profile.merge(&INBOUNDS.clone());

    let profile_routing = profile
        .get("routing")
        .cloned()
        .unwrap_or_else(|| json!({}));

    if let Some(base_routing) = ROUTING.get("routing").cloned() { // always true but it's better to double-check
        profile
            .as_object_mut()
            .unwrap()
            .insert("routing".to_string(), base_routing);
    }

    profile.merge(&json!({ "routing": profile_routing }));

    Ok(profile)
}

pub fn generate_config_from_profile(
    path: Option<&str>
) -> anyhow::Result<Value> {
    let profile = get_config_from_profile(path)?;

    std::fs::write("/etc/xray/config.json",
                   serde_json::to_string_pretty(&profile)?)?;

    Ok(profile)
}
