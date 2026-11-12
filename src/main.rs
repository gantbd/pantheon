use chronos_client::WireClientV1;
use crossterm::event::{self, Event};
use workbench_tui::{Focus, RoutedInput, draw, route_key};

struct UnavailableChronos;

impl WireClientV1 for UnavailableChronos {
    type Error = ();

    fn request(
        &mut self,
        _: chronos_protocol::Request,
    ) -> Result<chronos_protocol::Response, Self::Error> {
        Err(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut chronos = chronos_tui::Component::new(UnavailableChronos, false);
    let mut focus = Focus::Chronos;
    ratatui::run(|terminal| -> Result<(), Box<dyn std::error::Error>> {
        loop {
            chronos.poll();
            terminal.draw(|frame| {
                draw(
                    frame,
                    &chronos,
                    None,
                    focus,
                    "qualification shell; services intentionally unattached",
                )
            })?;
            let Event::Key(key) = event::read()? else {
                continue;
            };
            match route_key(key, focus) {
                RoutedInput::Exit => break,
                RoutedInput::SwitchFocus => focus = focus.other(),
                RoutedInput::Chronos(input) => {
                    let _ = chronos.update(input, true);
                }
                RoutedInput::Vellum(_) => {}
            }
        }
        Ok(())
    })
}
