use serde_lite::{intermediate, Deserialize, Intermediate, Map, Number, Serialize, Update};

use serde_lite_derive::{Deserialize, Serialize, Update};

#[test]
fn test_struct_deserialize() {
    let input = intermediate!({
        "number": 33,
        "hello": "hello",
    });

    #[derive(Deserialize)]
    struct OuterStruct {
        #[serde(flatten)]
        inner: InnerStruct,
    }

    #[derive(Deserialize)]
    struct InnerStruct {
        number: u32,
        #[serde(rename = "hello")]
        greetings: String,
        #[serde(default)]
        foo: Option<String>,
    }

    let output = OuterStruct::deserialize(&input).unwrap();

    assert_eq!(output.inner.number, 33);
    assert_eq!(output.inner.greetings.as_str(), "hello");
    assert!(output.inner.foo.is_none());

    assert!(OuterStruct::deserialize(&Intermediate::None).is_err());
}

#[test]
fn test_struct_update() {
    let input = intermediate!({
        "number": 33,
        "hello": "hi",
    });

    #[derive(Deserialize, Update)]
    struct OuterStruct {
        #[serde(flatten)]
        inner: InnerStruct,
    }

    #[derive(Deserialize, Update)]
    struct InnerStruct {
        number: u32,
        #[serde(rename = "hello")]
        greetings: String,
        year: u32,
    }

    let mut instance = OuterStruct {
        inner: InnerStruct {
            number: 0,
            greetings: String::new(),
            year: 2021,
        },
    };

    instance.update(&input).unwrap();

    assert_eq!(instance.inner.number, 33);
    assert_eq!(instance.inner.greetings.as_str(), "hi");
    assert_eq!(instance.inner.year, 2021);

    assert!(instance.update(&Intermediate::None).is_err());
}

#[test]
fn test_tuple_struct_deserialize() {
    let input1 = intermediate!(10);
    let input2 = intermediate!([10]);

    #[derive(Deserialize)]
    struct SingleElementTuple(u32);

    let output1 = SingleElementTuple::deserialize(&input1).unwrap();

    assert_eq!(output1.0, 10);

    assert!(SingleElementTuple::deserialize(&input2).is_err());

    #[derive(Deserialize)]
    struct MultiElementStruct(u32, String);

    let input = intermediate!([32, "hello"]);

    let output = MultiElementStruct::deserialize(&input).unwrap();

    assert_eq!(output.0, 32);
    assert_eq!(output.1.as_str(), "hello");

    assert!(MultiElementStruct::deserialize(&Intermediate::Array(vec![])).is_err());
}

#[test]
fn test_tuple_struct_update() {
    let input1 = intermediate!(10);
    let input2 = intermediate!([20]);

    #[derive(Deserialize, Update)]
    struct SingleElementTuple(u32);

    let mut instance = SingleElementTuple(0);

    instance.update(&input1).unwrap();

    assert_eq!(instance.0, 10);

    instance.update(&input2).unwrap();

    assert_eq!(instance.0, 20);

    #[derive(Deserialize, Update)]
    struct MultiElementStruct(u32, String);

    let input = intermediate!([32, "hello"]);

    let mut instance = MultiElementStruct(0, String::new());

    instance.update(&input).unwrap();

    assert_eq!(instance.0, 32);
    assert_eq!(instance.1.as_str(), "hello");

    assert!(instance.update(&Intermediate::Array(vec![])).is_err());
}

#[test]
fn test_empty_struct_deserialize() {
    #[derive(Deserialize)]
    struct UnitStruct;

    #[derive(Deserialize)]
    struct EmptyStruct {}

    #[derive(Deserialize)]
    struct EmptyTupleStruct();

    assert!(UnitStruct::deserialize(&Intermediate::None).is_ok());
    assert!(EmptyStruct::deserialize(&Intermediate::None).is_ok());
    assert!(EmptyTupleStruct::deserialize(&Intermediate::None).is_ok());
}

#[test]
fn test_empty_struct_update() {
    #[derive(Deserialize, Update)]
    struct UnitStruct;

    #[derive(Deserialize, Update)]
    struct EmptyStruct {}

    #[derive(Deserialize, Update)]
    struct EmptyTupleStruct();

    let mut unit = UnitStruct;
    let mut empty = EmptyStruct {};
    let mut empty_tuple = EmptyTupleStruct();

    assert!(unit.update(&Intermediate::None).is_ok());
    assert!(empty.update(&Intermediate::None).is_ok());
    assert!(empty_tuple.update(&Intermediate::None).is_ok());
}

#[test]
fn test_enum_deserialize() {
    let input1 = intermediate!("Variant1");

    let input2 = intermediate!({
        "Variant1": null,
    });

    let input3 = intermediate!({
        "variant2": 20,
    });

    let input4 = intermediate!({
        "variant2": [20],
    });

    let input5 = intermediate!({
        "Variant3": {
            "field1": 10,
            "field2": "hello",
        },
    });

    let input6 = intermediate!("foo");

    let input7 = intermediate!(null);

    let input8 = intermediate!({
        "Variant4": [],
    });

    let input9 = intermediate!({
        "Variant4": [123],
    });

    #[derive(Deserialize)]
    enum TestEnum {
        Variant1,
        #[serde(rename = "variant2")]
        Variant2(u32),
        Variant3 {
            field1: u32,
            field2: String,
        },
        Variant4(Vec<u32>),
    }

    let output1 = TestEnum::deserialize(&input1).unwrap();
    let output2 = TestEnum::deserialize(&input2).unwrap();
    let output3 = TestEnum::deserialize(&input3).unwrap();
    let output5 = TestEnum::deserialize(&input5).unwrap();
    let output8 = TestEnum::deserialize(&input8).unwrap();
    let output9 = TestEnum::deserialize(&input9).unwrap();

    assert!(matches!(output1, TestEnum::Variant1));
    assert!(matches!(output2, TestEnum::Variant1));

    if let TestEnum::Variant2(n) = output3 {
        assert_eq!(n, 20);
    } else {
        panic!("output3 test failed");
    }

    if let TestEnum::Variant3 { field1, field2 } = output5 {
        assert_eq!(field1, 10);
        assert_eq!(field2.as_str(), "hello");
    } else {
        panic!("output4 test failed");
    }

    assert!(TestEnum::deserialize(&input4).is_err());
    assert!(TestEnum::deserialize(&input6).is_err());
    assert!(TestEnum::deserialize(&input7).is_err());

    if let TestEnum::Variant4(arr) = output8 {
        assert_eq!(arr.as_slice(), &[][..]);
    } else {
        panic!("output8 test failed");
    }

    if let TestEnum::Variant4(arr) = output9 {
        assert_eq!(arr.as_slice(), &[123][..]);
    } else {
        panic!("output9 test failed");
    }
}

#[test]
fn test_enum_update() {
    let input1 = intermediate!("Variant1");

    let input2 = intermediate!({
        "Variant1": null,
    });

    let input3 = intermediate!({
        "variant2": 20,
    });

    let input4 = intermediate!({
        "variant2": [40],
    });

    let input5 = intermediate!({
        "Variant3": {
            "field1": 20,
        },
    });

    let input6 = intermediate!("foo");

    let input7 = intermediate!(null);

    #[derive(Deserialize, Update)]
    enum TestEnum {
        Variant1,
        #[serde(rename = "variant2")]
        Variant2(u32),
        Variant3 {
            field1: u32,
            field2: String,
        },
    }

    let mut instance = TestEnum::Variant2(0);
    instance.update(&input1).unwrap();
    assert!(matches!(instance, TestEnum::Variant1));

    let mut instance = TestEnum::Variant2(0);
    instance.update(&input2).unwrap();
    assert!(matches!(instance, TestEnum::Variant1));

    let mut instance = TestEnum::Variant1;

    instance.update(&input3).unwrap();

    if let TestEnum::Variant2(n) = &instance {
        assert_eq!(*n, 20);
    } else {
        panic!("test failed");
    }

    instance.update(&input4).unwrap();

    if let TestEnum::Variant2(n) = instance {
        assert_eq!(n, 40);
    } else {
        panic!("test failed");
    }

    let mut instance = TestEnum::Variant1;

    assert!(instance.update(&input5).is_err());

    let mut instance = TestEnum::Variant3 {
        field1: 0,
        field2: String::new(),
    };

    instance.update(&input5).unwrap();

    if let TestEnum::Variant3 { field1, field2 } = instance {
        assert_eq!(field1, 20);
        assert_eq!(field2.as_str(), "");
    } else {
        panic!("test failed");
    }

    let mut instance = TestEnum::Variant1;

    assert!(instance.update(&input6).is_err());
    assert!(instance.update(&input7).is_err());
}

#[test]
fn test_internally_tagged_enum_deserialize_and_update() {
    #[derive(Deserialize, Update)]
    struct OuterStruct {
        field: TestEnum,
    }

    #[derive(Deserialize, Update)]
    #[serde(tag = "type")]
    enum TestEnum {
        Variant1,
        Variant2(InnerStruct),
    }

    #[derive(Deserialize, Update)]
    struct InnerStruct {
        field1: u32,
        field2: String,
    }

    let input = intermediate!({
        "field": {
            "type": "Variant2",
            "field1": 10,
            "field2": "hello",
        },
    });

    let mut instance = OuterStruct::deserialize(&input).unwrap();

    if let TestEnum::Variant2(inner) = &instance.field {
        assert_eq!(inner.field1, 10);
        assert_eq!(inner.field2.as_str(), "hello");
    } else {
        panic!("test failed");
    }

    let input = intermediate!({
        "field": {
            "type": "Variant2",
            "field2": "world",
        },
    });

    instance.update(&input).unwrap();

    if let TestEnum::Variant2(inner) = &instance.field {
        assert_eq!(inner.field1, 10);
        assert_eq!(inner.field2.as_str(), "world");
    } else {
        panic!("test failed");
    }
}

#[test]
fn test_adjacently_tagged_enum_deserialize_and_update() {
    #[derive(Deserialize, Update)]
    struct OuterStruct {
        field: TestEnum,
    }

    #[derive(Deserialize, Update)]
    #[serde(tag = "type", content = "content")]
    enum TestEnum {
        Variant1,
        Variant2(InnerStruct),
    }

    #[derive(Deserialize, Update)]
    struct InnerStruct {
        field1: u32,
        field2: String,
    }

    let input = intermediate!({
        "field": {
            "type": "Variant2",
            "content": {
                "field1": 10,
                "field2": "hello",
            },
        },
    });

    let mut instance = OuterStruct::deserialize(&input).unwrap();

    if let TestEnum::Variant2(inner) = &instance.field {
        assert_eq!(inner.field1, 10);
        assert_eq!(inner.field2.as_str(), "hello");
    } else {
        panic!("test failed");
    }

    let input = intermediate!({
        "field": {
            "type": "Variant2",
            "content": {
                "field2": "world",
            },
        },
    });

    instance.update(&input).unwrap();

    if let TestEnum::Variant2(inner) = &instance.field {
        assert_eq!(inner.field1, 10);
        assert_eq!(inner.field2.as_str(), "world");
    } else {
        panic!("test failed");
    }
}

#[test]
fn test_struct_serialize() {
    #[derive(Serialize)]
    struct OuterTestStruct {
        field1: u32,
        field2: InnerTestStruct,
        #[serde(flatten)]
        field3: InnerTestStruct,
    }

    #[derive(Serialize)]
    struct InnerTestStruct {
        inner1: bool,
        inner2: String,
    }

    let instance = OuterTestStruct {
        field1: 10,
        field2: InnerTestStruct {
            inner1: true,
            inner2: String::from("hello"),
        },
        field3: InnerTestStruct {
            inner1: false,
            inner2: String::from("world"),
        },
    };

    let data = instance.serialize().unwrap();

    let map = data.as_map().unwrap();

    assert_eq!(map.len(), 4);
    assert_eq!(get_unsigned_int_field(map, "field1"), 10);

    let field2 = get_map_field(map, "field2");
    assert_eq!(field2.len(), 2);
    assert_eq!(get_bool_field(field2, "inner1"), true);
    assert_eq!(get_str_field(field2, "inner2"), "hello");

    assert_eq!(get_bool_field(map, "inner1"), false);
    assert_eq!(get_str_field(map, "inner2"), "world");
}

#[test]
fn test_externally_tagged_enum_serialize() {
    #[derive(Serialize)]
    struct OuterTestStruct {
        field5: ExternallyTaggedEnum,
        field6: ExternallyTaggedEnum,
        field7: ExternallyTaggedEnum,
        field8: ExternallyTaggedEnum,
        field9: ExternallyTaggedEnum,
        field10: ExternallyTaggedEnum,
        field11: ExternallyTaggedEnum,
    }

    #[derive(Serialize)]
    struct InnerTestStruct {
        inner1: bool,
        inner2: String,
    }

    #[derive(Serialize)]
    enum ExternallyTaggedEnum {
        Variant1,
        #[serde(rename = "v2")]
        Variant2(u32),
        Variant3(u32, String),
        Variant4 {
            field1: u32,
            #[serde(flatten)]
            field2: InnerTestStruct,
        },
        Variant5(InnerTestStruct),
        Variant6(Vec<u32>),
    }

    let instance = OuterTestStruct {
        field5: ExternallyTaggedEnum::Variant1,
        field6: ExternallyTaggedEnum::Variant2(20),
        field7: ExternallyTaggedEnum::Variant3(30, String::from("sss")),
        field8: ExternallyTaggedEnum::Variant4 {
            field1: 40,
            field2: InnerTestStruct {
                inner1: true,
                inner2: String::from("zzz"),
            },
        },
        field9: ExternallyTaggedEnum::Variant5(InnerTestStruct {
            inner1: false,
            inner2: String::from("abc"),
        }),
        field10: ExternallyTaggedEnum::Variant6(vec![]),
        field11: ExternallyTaggedEnum::Variant6(vec![50]),
    };

    let data = instance.serialize().unwrap();

    let map = data.as_map().unwrap();

    assert_eq!(get_str_field(map, "field5"), "Variant1");

    let field6 = get_map_field(map, "field6");
    assert_eq!(field6.len(), 1);
    assert_eq!(get_unsigned_int_field(field6, "v2"), 20);

    let field7 = get_map_field(map, "field7");
    assert_eq!(field7.len(), 1);
    let arr = get_array_field(field7, "Variant3");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0].as_u64().unwrap().unwrap(), 30);
    assert_eq!(arr[1].as_str().unwrap(), "sss");

    let field8 = get_map_field(map, "field8");
    assert_eq!(field8.len(), 1);
    let inner = get_map_field(field8, "Variant4");
    assert_eq!(inner.len(), 3);
    assert_eq!(get_unsigned_int_field(inner, "field1"), 40);
    assert_eq!(get_bool_field(inner, "inner1"), true);
    assert_eq!(get_str_field(inner, "inner2"), "zzz");

    let field9 = get_map_field(map, "field9");
    assert_eq!(field9.len(), 1);
    let inner = get_map_field(field9, "Variant5");
    assert_eq!(inner.len(), 2);
    assert_eq!(get_bool_field(inner, "inner1"), false);
    assert_eq!(get_str_field(inner, "inner2"), "abc");

    let field10 = get_map_field(map, "field10");
    assert_eq!(field10.len(), 1);
    let arr = get_array_field(field10, "Variant6");
    assert_eq!(arr.len(), 0);

    let field11 = get_map_field(map, "field11");
    assert_eq!(field11.len(), 1);
    let arr = get_array_field(field11, "Variant6");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].as_u64().unwrap().unwrap(), 50);
}

#[test]
fn test_internally_tagged_enum_serialize() {
    #[derive(Serialize)]
    struct OuterTestStruct {
        field4: EnumContainerStruct,
        field10: InternallyTaggedEnum,
        field11: InternallyTaggedEnum,
        field12: InternallyTaggedEnum,
    }

    #[derive(Serialize)]
    struct InnerTestStruct {
        inner1: bool,
        inner2: String,
    }

    #[derive(Serialize)]
    struct EnumContainerStruct {
        e1: InternallyTaggedEnum,
        #[serde(flatten)]
        e2: InternallyTaggedEnum,
    }

    #[derive(Serialize)]
    #[serde(tag = "variant")]
    enum InternallyTaggedEnum {
        Variant1,
        #[serde(rename = "v2")]
        Variant2(u32),
        Variant3(u32, String),
        Variant4 {
            field1: u32,
            #[serde(flatten)]
            field2: InnerTestStruct,
        },
        Variant5(InnerTestStruct),
    }

    let instance = OuterTestStruct {
        field4: EnumContainerStruct {
            e1: InternallyTaggedEnum::Variant4 {
                field1: 30,
                field2: InnerTestStruct {
                    inner1: true,
                    inner2: String::from("foo"),
                },
            },
            e2: InternallyTaggedEnum::Variant5(InnerTestStruct {
                inner1: false,
                inner2: String::from("bar"),
            }),
        },
        field10: InternallyTaggedEnum::Variant1,
        field11: InternallyTaggedEnum::Variant4 {
            field1: 50,
            field2: InnerTestStruct {
                inner1: true,
                inner2: String::from("xyz"),
            },
        },
        field12: InternallyTaggedEnum::Variant5(InnerTestStruct {
            inner1: true,
            inner2: String::from("qwerty"),
        }),
    };

    let data = instance.serialize().unwrap();

    let map = data.as_map().unwrap();

    let field4 = get_map_field(map, "field4");
    assert_eq!(field4.len(), 4);
    let e1 = get_map_field(field4, "e1");
    assert_eq!(e1.len(), 4);
    assert_eq!(get_str_field(e1, "variant"), "Variant4");
    assert_eq!(get_unsigned_int_field(e1, "field1"), 30);
    assert_eq!(get_bool_field(e1, "inner1"), true);
    assert_eq!(get_str_field(e1, "inner2"), "foo");
    assert_eq!(get_str_field(field4, "variant"), "Variant5");
    assert_eq!(get_bool_field(field4, "inner1"), false);
    assert_eq!(get_str_field(field4, "inner2"), "bar");

    let field10 = get_map_field(map, "field10");
    assert_eq!(field10.len(), 1);
    assert_eq!(get_str_field(field10, "variant"), "Variant1");

    let field11 = get_map_field(map, "field11");
    assert_eq!(field11.len(), 4);
    assert_eq!(get_str_field(field11, "variant"), "Variant4");
    assert_eq!(get_unsigned_int_field(field11, "field1"), 50);
    assert_eq!(get_bool_field(field11, "inner1"), true);
    assert_eq!(get_str_field(field11, "inner2"), "xyz");

    let field12 = get_map_field(map, "field12");
    assert_eq!(field12.len(), 3);
    assert_eq!(get_str_field(field12, "variant"), "Variant5");
    assert_eq!(get_bool_field(field12, "inner1"), true);
    assert_eq!(get_str_field(field12, "inner2"), "qwerty");

    let instance = InternallyTaggedEnum::Variant2(10);
    assert!(instance.serialize().is_err());

    let instance = InternallyTaggedEnum::Variant3(20, String::from("foo"));
    assert!(instance.serialize().is_err());
}

#[test]
fn test_adjacently_tagged_enum_serialize() {
    #[derive(Serialize)]
    struct OuterTestStruct {
        field13: AdjacentlyTaggedEnum,
        field14: AdjacentlyTaggedEnum,
    }

    #[derive(Serialize)]
    #[serde(tag = "variant", content = "content")]
    enum AdjacentlyTaggedEnum {
        Variant1,
        Variant2(u32, String),
    }

    let instance = OuterTestStruct {
        field13: AdjacentlyTaggedEnum::Variant1,
        field14: AdjacentlyTaggedEnum::Variant2(60, String::from("asdf")),
    };

    let data = instance.serialize().unwrap();

    let map = data.as_map().unwrap();

    let field13 = get_map_field(map, "field13");
    assert_eq!(field13.len(), 2);
    assert_eq!(get_str_field(field13, "variant"), "Variant1");
    assert!(field13.get("content").unwrap().is_none());

    let field14 = get_map_field(map, "field14");
    assert_eq!(field14.len(), 2);
    assert_eq!(get_str_field(field14, "variant"), "Variant2");
    let arr = get_array_field(field14, "content");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0].as_u64().unwrap().unwrap(), 60);
    assert_eq!(arr[1].as_str().unwrap(), "asdf");
}

#[test]
#[allow(unused_variables)]
fn test_skip_deserializing() {
    #[derive(Deserialize, Update)]
    struct TestStruct {
        field1: u32,
        #[serde(skip)]
        field2: u32,
        #[serde(skip_deserializing)]
        field3: u32,
    }

    let mut map = Map::new();
    map.insert(
        String::from("field1"),
        Intermediate::Number(Number::UnsignedInt(10)),
    );
    map.insert(
        String::from("field2"),
        Intermediate::Number(Number::UnsignedInt(20)),
    );
    map.insert(
        String::from("field3"),
        Intermediate::Number(Number::UnsignedInt(30)),
    );
    let input = Intermediate::Map(map);

    let mut instance = TestStruct::deserialize(&input).unwrap();

    assert_eq!(instance.field1, 10);
    assert_eq!(instance.field2, u32::default());
    assert_eq!(instance.field3, u32::default());

    instance.update(&input).unwrap();

    assert_eq!(instance.field1, 10);
    assert_eq!(instance.field2, u32::default());
    assert_eq!(instance.field3, u32::default());
}

#[test]
#[allow(unused_variables)]
fn test_skip_serializing() {
    #[derive(Serialize)]
    struct TestStruct {
        field1: u32,
        #[serde(skip)]
        field2: u32,
        #[serde(skip_serializing)]
        field3: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        field4: Option<u32>,
    }

    let mut instance = TestStruct {
        field1: 1,
        field2: 2,
        field3: 3,
        field4: Some(4),
    };

    let data = instance.serialize().unwrap();
    let map = data.as_map().unwrap();
    assert_eq!(map.len(), 2);
    assert_eq!(get_unsigned_int_field(map, "field1"), 1);
    assert_eq!(get_unsigned_int_field(map, "field4"), 4);

    instance.field4 = None;

    let data = instance.serialize().unwrap();
    let map = data.as_map().unwrap();
    assert_eq!(map.len(), 1);
    assert_eq!(get_unsigned_int_field(map, "field1"), 1);
}

/// Helper.
fn get_map_field<'a>(map: &'a Map, name: &str) -> &'a Map {
    map.get(name).unwrap().as_map().unwrap()
}

/// Helper.
fn get_array_field<'a>(map: &'a Map, name: &str) -> &'a [Intermediate] {
    map.get(name).unwrap().as_array().unwrap()
}

/// Helper.
fn get_bool_field(map: &Map, name: &str) -> bool {
    map.get(name).unwrap().as_bool().unwrap()
}

/// Helper.
fn get_unsigned_int_field(map: &Map, name: &str) -> u64 {
    map.get(name).unwrap().as_u64().unwrap().unwrap()
}

/// Helper.
fn get_str_field<'a>(map: &'a Map, name: &str) -> &'a str {
    map.get(name).unwrap().as_str().unwrap()
}
