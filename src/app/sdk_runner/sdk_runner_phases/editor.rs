use super::super::SdkApp;

pub fn run(app: &mut SdkApp) {
    // Feed live data into dashboard panels
    app.editor_shell.update_dashboards(&app.engine);

    // Apply any pending inspector edits
    app.editor_shell.apply_inspector_edits(&mut app.engine);
}
