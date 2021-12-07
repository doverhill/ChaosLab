extern crate chaos;
use chaos::{ process::Process, service::Service };

struct TestState {
    counter: u32
}

// this function will build the render list and send it to the tornado server
// it will allocate a unique id for every component
// event handlers will be stored in TornadoContext mapped to the component id and invoked automatically
fn render(context: TornadoContext<TestState>) {
    render!(
        <Window Title=format!("Editor - {}", context.state.counter)>
            <Button Text="Click me" OnClick=handle_click>
        </Window>
    );
}

fn handle_click(context: TornadoContext<TestState>, event: ClickEvent) {
    context.state.counter++;
    render(context);
}

fn main() {
    // to be nice, set a name for our application
    Process::set_info("Application.GuiTest").unwrap();

    // attempt to connect to the vfs service
    match Service::connect("tornado", None, None, None, 4096) {
        Ok(channel_wrap) => {
            let state = TestState { counter: 0 };
            let context = TornadoContext::new::<TestState>(channel_wrap, state);
            render(context);
            Process::run();
        },
        Err(error) => {
            Process::emit_error(error, "Failed to connect to Tornado service").unwrap();
        }
    }

    // this is needed for now at the end of every program to clean up correctly
    Process::end();
}
