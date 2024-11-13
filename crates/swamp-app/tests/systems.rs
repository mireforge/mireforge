use swamp_app::prelude::*;

#[derive(Debug, Resource)]
pub struct TestResource {
    pub value: i32,
}

#[derive(Debug, Resource)]
pub struct AnotherTestResource {
    pub value: Option<i32>,
}

#[derive(Debug, Message)]
pub struct TestMessage {
    pub value: i32,
}

fn test_system(mut res: ReM<TestResource>) {
    res.value = -1;
}

const VALUE_SET_BY_SYSTEM_2: i32 = 99;

#[allow(clippy::needless_pass_by_value)]
fn test_system_2(msg: Msg<TestMessage>, mut res: ReM<TestResource>) {
    if msg.is_empty_current() && res.value == -1 {
        res.value = VALUE_SET_BY_SYSTEM_2;
    }
}

#[allow(clippy::needless_pass_by_value)]
fn test_system_3(msg: Msg<TestMessage>, mut res: ReM<TestResource>, res2: Re<AnotherTestResource>) {
    if let Some(v) = res2.value {
        if msg.is_empty_current() && res.value == VALUE_SET_BY_SYSTEM_2 {
            res.value = v;
        }
    }
}

#[test]
fn test_systems() {
    const EXPECTED_VALUE: i32 = 42;

    let mut app = App::new();
    app.insert_resource(TestResource { value: 0 });
    app.insert_resource(AnotherTestResource {
        value: Some(EXPECTED_VALUE),
    });
    app.create_message_type::<TestMessage>();

    app.add_system(UpdatePhase::Update, test_system_3); // this will run last
    app.add_system(UpdatePhase::First, test_system);
    app.add_system(UpdatePhase::First, test_system_2);

    app.update();

    assert_eq!(app.resource::<TestResource>().value, EXPECTED_VALUE);
}
