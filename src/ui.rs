// src/ui.rs
//
// Onboarding screen + a stub for the main window.
// The onboarding card is centered and intentionally minimal: title, input,
// "Show key" toggle, primary action, status line, and a help link.

use crate::app::{App, OnboardingStatus, Screen};
use eframe::egui;

pub fn render(app: &mut App, ctx: &egui::Context) {
    app.poll_validation();
    // Re-paint while validating so the spinner animates.
    if matches!(
        &app.screen,
        Screen::Onboarding(s) if matches!(s.status, OnboardingStatus::Validating)
    ) {
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }

    match &mut app.screen {
        Screen::Onboarding(_) => render_onboarding(app, ctx),
        Screen::Main(_) => render_main_stub(app, ctx),
    }
}

fn render_onboarding(app: &mut App, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(80.0);

            ui.heading("Welcome to Krater Chat");
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("Connect your OpenRouter API key to get started.")
                    .color(egui::Color32::GRAY),
            );
            ui.add_space(28.0);

            // Card
            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(20.0))
                .show(ui, |ui| {
                    ui.set_max_width(440.0);

                    let Screen::Onboarding(state) = &mut app.screen else {
                        return;
                    };

                    ui.label("OpenRouter API key");
                    ui.add_space(4.0);

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut state.key_input)
                            .password(!state.show_key)
                            .hint_text("sk-or-v1-…")
                            .desired_width(f32::INFINITY),
                    );

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut state.show_key, "Show key");
                        ui.add_space(8.0);
                        ui.hyperlink_to("Get a key", "https://openrouter.ai/keys");
                    });

                    ui.add_space(14.0);

                    let validating = matches!(state.status, OnboardingStatus::Validating);

                    let submit_clicked = ui
                        .add_enabled(
                            !validating,
                            egui::Button::new(if validating {
                                "Validating…"
                            } else {
                                "Continue"
                            })
                            .min_size(egui::vec2(120.0, 32.0)),
                        )
                        .clicked();

                    let enter_pressed = response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        && !validating;

                    if submit_clicked || enter_pressed {
                        app.start_validation();
                        return;
                    }

                    ui.add_space(10.0);
                    match &state.status {
                        OnboardingStatus::Idle => {}
                        OnboardingStatus::Validating => {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label("Checking your key with OpenRouter…");
                            });
                        }
                        OnboardingStatus::Error(msg) => {
                            ui.colored_label(egui::Color32::from_rgb(220, 80, 80), msg);
                        }
                    }
                });

            ui.add_space(20.0);
            ui.label(
                egui::RichText::new(
                    "Your key is stored in Windows Credential Manager and never written to disk in plain text.",
                )
                .small()
                .color(egui::Color32::GRAY),
            );
        });
    });
}

fn render_main_stub(_app: &mut App, ctx: &egui::Context) {
    // Real chat UI (left history pane, model picker, "+" attachments,
    // temporary-chat toggle) is the next step. This stub just confirms
    // the auth flow worked end to end.
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(120.0);
            ui.heading("Connected");
            ui.add_space(8.0);
            ui.label("API key validated and securely cached.");
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Chat UI will be wired up in the next step.")
                    .color(egui::Color32::GRAY),
            );
        });
    });
}