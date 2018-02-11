use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use ::serde_json;
use serde_json::Value;

use super::{DATA, Ecs};

pub struct Data();

impl DATA for Data {
    fn argv_new(&self, region: String) -> Vec<String> {
        let mut argv = self.argv_new_base(region);
        argv.push("net_tcpconnection".to_owned());

        argv.push("StartTime".to_owned());
        unsafe {
            argv.push(::BASESTAMP.to_string());
        }

        argv.push("EndTime".to_owned());
        unsafe {
            argv.push((::BASESTAMP + ::INTERVAL).to_string());
        }

        argv
    }

    fn insert(&self, holder: &Arc<Mutex<HashMap<String, Ecs>>>, data: Vec<u8>) {
        let v: Value = serde_json::from_slice(&data).unwrap_or(Value::Null);
        if Value::Null == v {
            return;
        }

        let body = &v["Datapoints"];
        for i in 0.. {
            if Value::Null == body[i] {
                break;
            } else {
                /* take TCP_TOTAL only! */
                if let Value::String(ref s) = body[i]["state"] {
                    if "TCP_TOTAL" != s { continue; }
                } else { continue; }

                let mut ecsid;
                let mut ts;

                if let Value::String(ref id) = body[i]["instanceId"] {
                    ecsid = id;
                } else { continue; }

                if let Value::Number(ref t) = body[i]["timestamp"] {
                    if let Some(t) = t.as_u64() {
                        ts = t;
                    } else { continue; }
                } else { continue; }

                if let Some(ecs) = holder.lock().unwrap().get_mut(ecsid) {
                    /* align with 15s */
                    if let Some(inner) = ecs.data.get_mut(&(ts / 15000 * 15000)) {
                        if let Value::Number(ref v) = body[i]["Average"] {
                            if let Some(v) = v.as_u64() {
                                inner.tcp = v as i32;
                            } else { continue; }
                        } else { continue; }
                    } else { continue; }
                }
            }
        }
    }
}
