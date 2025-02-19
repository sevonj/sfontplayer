use eframe::egui::{vec2, Align, Button, Label, Layout, Response, RichText, Ui, Vec2, Widget};

pub fn circle_button<S>(title: S, ui: &mut Ui) -> Response
where
    String: From<S>,
{
    ui.add(
        Button::new(String::from(title))
            .corner_radius(32.)
            .min_size(vec2(20., 20.)),
    )
}

pub fn collapse_button(open: &mut bool) -> impl Widget + '_ {
    move |ui: &mut Ui| {
        let icon = if *open { "⏷" } else { "⏵" };

        let response = ui
            .with_layout(Layout::left_to_right(Align::Max), |ui| {
                ui.set_height(16.);
                ui.add(
                    Button::new(RichText::new(icon).size(16.))
                        .frame(false)
                        .min_size(Vec2::new(8., 0.)),
                )
            })
            .inner;
        if response.clicked() {
            *open = !*open;
        }
        response
    }
}

pub fn subheading<S>(title: S) -> Label
where
    String: From<S>,
{
    Label::new(RichText::new(String::from(title)).size(14.)).selectable(false)
}
