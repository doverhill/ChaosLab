extern crate chaos;
use chaos::{ process::Process, service::Service };

// this function will build the render list and send it to the tornado server
// it will allocate a unique id for every component
// event handlers will be stored in TornadoContext mapped to the component id and invoked automatically
fn render(context: TornadoContext<EditorState>) {
    render!(
        <Window Title=format!("Editor - {}", context.state.filename)>
            <GridLayout>
                <GridLayoutColumn SizeMode="ContentMaximum" />
                <GridLayoutColumn SizeMode="Fraction" Fraction=1 />
                <GridLayoutRow SizeMode="Fraction" Fraction=1 />
                <GridLayoutRow SizeMode="ContentMinimum" />
                
                <GridCell Row=0 Column=0>
                    <Label Text="File selector" />
                </GridCell>
                <GridCell Row=0 Column=1>
                    <TextEditor Text=context.state.Text OnCursorMove=handle_cursor_move OnEdit=handle_edit />
                </GridCell>
                <GridCell Row=1 Column=0 ColumnSpan=2>
                    <Label Text=format!("Row {} Column {}", context.state.RowNumber, context.state.ColumnNumber) />
                </GridCell>
            </GridLayout>
        </Window>
    );
}

fn handle_cursor_move(context: TornadoContext<EditorState>, event: TextEditorCursorMoveEvent) {
    context.state.RowNumber = event.RowNumber;
    context.state.ColumnNumber = event.ColumnNumber;
    render(context);
}

fn handle_edit(context: TornadoContext<EditorState>, event: TextEditorEditEvent) {
    context.state.Text = event.Text;
    // no need to render, just store the new value
}

fn main() {
    // to be nice, set a name for our application
    Process::set_info("Application.Editor").unwrap();

    // attempt to connect to the vfs service
    match Service::connect("tornado", None, None, None, 4096) {
        Ok(channel_wrap) => {
            let state = EditorState::new("This is some text\n...and another line", 0, 0);
            let context = TornadoContext::new::<EditorState>(channel_wrap, state);
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
