use std::sync::{Mutex, OnceLock};

use serde_json::json;
use wasmtime::{
    component::{Component, Instance, Linker, Type},
    Config, Engine, Store,
};
use wasmtime_component_serde::{deserialize_val, serialize_val};

#[test]
fn test_round_trips() {
    assert_round_trip("bools", json!([true, false]));

    assert_round_trip("sints", json!([i8::MIN, i16::MIN, i32::MIN, i64::MIN]));
    assert_round_trip("uints", json!([u8::MAX, u16::MAX, u32::MAX, u64::MAX]));

    assert_round_trip("floats", json!([1.5, 1.5]));
    assert_round_trip("floats", json!(["Infinity", "Infinity"]));
    assert_round_trip("floats", json!(["-Infinity", "-Infinity"]));
    assert_round_trip("floats", json!(["NaN", "NaN"]));

    assert_round_trip("options", json!([null, null]));
    assert_round_trip("options", json!([1, {"value": null}]));
    assert_round_trip("options", json!([1, {"value": 2}]));

    assert_round_trip("list-chars", json!([]));
    assert_round_trip("list-chars", json!(["x", "☃"]));
    assert_round_trip("list-strings", json!(["xyz", "☃☃☃"]));

    assert_round_trip("result-ok-only", json!({"result": 1}));
    assert_round_trip("result-ok-only", json!({"error": null}));
    assert_round_trip("result-err-only", json!({"result": null}));
    assert_round_trip("result-err-only", json!({"error": -1}));
    assert_round_trip("result-no-payloads", json!({"result": null}));
    assert_round_trip("result-no-payloads", json!({"error": null}));
    assert_round_trip("result-both-payloads", json!({"result": 1}));
    assert_round_trip("result-both-payloads", json!({"error": -1}));

    assert_round_trip("record", json!({"required": 1}));
    assert_round_trip("record", json!({"required": 1, "optional": 1}));

    assert_round_trip("variant", json!({"without-payload": null}));
    assert_round_trip("variant", json!({"with-payload": 1}));

    assert_round_trip("enum", json!("first"));
    assert_round_trip("enum", json!("second"));

    assert_round_trip("flags", json!([]));
    assert_round_trip("flags", json!(["write"]));
    assert_round_trip("flags", json!(["read", "write"]));
}

fn assert_round_trip(type_name: &str, json: serde_json::Value) {
    let ty = get_type(type_name);
    let deserialized = deserialize_val(&json, &ty).unwrap();
    let serialized_json = serialize_val(serde_json::value::Serializer, &deserialized).unwrap();
    assert_eq!(serialized_json, json);
}

fn get_type(name: &str) -> Type {
    static INSTANCE_AND_STORE: OnceLock<(Instance, Mutex<Store<()>>)> = OnceLock::new();
    let (instance, store) = INSTANCE_AND_STORE.get_or_init(|| {
        let engine = Engine::new(Config::new().wasm_component_model(true)).expect("engine");
        let component = Component::from_file(&engine, "tests/types.wasm").expect("component");
        let linker = Linker::new(&engine);
        let mut store = Store::new(&engine, ());
        let instance = linker
            .instantiate(&mut store, &component)
            .expect("instance");
        (instance, Mutex::new(store))
    });
    let mut store = store.lock().unwrap();
    let func = instance
        .exports(&mut *store)
        .root()
        .func(name)
        .unwrap_or_else(|| panic!("export func named {name:?}"));
    func.results(&*store)[0].clone()
}
