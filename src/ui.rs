use crate::app::{App, Message, OnboardingStatus, Role, Screen, AVAILABLE_MODELS, DEFAULT_MODEL};
use eframe::egui;
use std::time::Duration;
use uuid::Uuid;

const FADE_DURATION: Duration = Duration::from_millis(300);

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
        Screen::Main(_) => render_main(app, ctx),
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

fn render_main(app: &mut App, ctx: &egui::Context) {
    // Poll for a completed assistant response before drawing this frame.
    app.poll_pending();

    // Snapshot a few fields we'll need outside the &mut state borrow.
    let (temporary_mode, has_pending, active_model, has_fading_message) = {
        let Screen::Main(state) = &app.screen else { return };
        let active_model = state
            .active_chat()
            .map(|c| c.model.clone())
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());
        let has_fading = state
            .active_chat()
            .map(|c| {
                c.messages.iter().any(|m| {
                    matches!(m.role, Role::Assistant)
                        && m.appeared_at.elapsed() < FADE_DURATION
                })
            })
            .unwrap_or(false);
        (
            state.temporary_mode,
            state.pending.is_some(),
            active_model,
            has_fading,
        )
    };

    // === TOP BAR =========================================================
    let mut new_model_choice: Option<String> = None;
    let mut new_temp_mode: Option<bool> = None;

    egui::TopBottomPanel::top("top_bar")
        .exact_height(44.0)
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                let active_label = AVAILABLE_MODELS
                    .iter()
                    .find(|(id, _)| *id == active_model)
                    .map(|(_, label)| *label)
                    .unwrap_or("Model");

                egui::ComboBox::from_id_salt("model_picker")
                    .selected_text(active_label)
                    .show_ui(ui, |ui| {
                        for (id, label) in AVAILABLE_MODELS {
                            let selected = active_model == *id;
                            if ui.selectable_label(selected, *label).clicked() {
                                new_model_choice = Some((*id).to_string());
                            }
                        }
                    });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut temp = temporary_mode;
                    if ui.toggle_value(&mut temp, "🕓 Temporary chat").changed() {
                        new_temp_mode = Some(temp);
                    }
                });
            });
        });

    if let Some(model) = new_model_choice {
        if let Screen::Main(state) = &mut app.screen {
            if let Some(chat) = state.active_chat_mut() {
                chat.model = model;
            }
        }
    }
    if let Some(on) = new_temp_mode {
        app.set_temporary_mode(on);
    }

    // === SIDEBAR =========================================================
    let mut select_request: Option<Uuid> = None;
    let mut delete_request: Option<Uuid> = None;
    let mut new_chat_request = false;

    egui::SidePanel::left("sidebar")
        .resizable(true)
        .default_width(220.0)
        .width_range(160.0..=360.0)
        .show(ctx, |ui| {
            let Screen::Main(state) = &app.screen else { return };

            ui.add_space(8.0);
            if ui
                .add_sized(
                    [ui.available_width(), 32.0],
                    egui::Button::new("➕  New chat"),
                )
                .clicked()
            {
                new_chat_request = true;
            }
            ui.add_space(8.0);
            ui.separator();

            if state.temporary_mode {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Temporary mode is on.\nThis chat will not be saved.")
                        .italics()
                        .color(egui::Color32::from_gray(160)),
                );
                return;
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                let active_id = state.active_chat_id;
                for chat in &state.chats {
                    let selected = active_id == Some(chat.id);
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(selected, truncate(&chat.title, 28))
                            .clicked()
                        {
                            select_request = Some(chat.id);
                        }
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if ui
                                    .small_button("🗑")
                                    .on_hover_text("Delete chat")
                                    .clicked()
                                {
                                    delete_request = Some(chat.id);
                                }
                            },
                        );
                    });
                }
            });
        });

    if new_chat_request {
        app.new_chat();
    }
    if let Some(id) = select_request {
        app.select_chat(id);
    }
    if let Some(id) = delete_request {
        app.delete_chat(id);
    }

    // === MAIN CHAT AREA ==================================================
    let mut send_request = false;

    egui::CentralPanel::default().show(ctx, |ui| {
        let Screen::Main(state) = &mut app.screen else { return };

        let input_height = 72.0;
        let avail = ui.available_size();

        // --- Message thread ---
        let messages_rect = egui::Rect::from_min_size(
            ui.min_rect().min,
            egui::vec2(avail.x, avail.y - input_height),
        );
        let mut messages_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(messages_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        render_messages(
            &mut messages_ui,
            state.active_chat().map(|c| c.messages.as_slice()),
        );

        // --- Input bar ---
        let input_rect = egui::Rect::from_min_size(
            egui::pos2(ui.min_rect().min.x, ui.min_rect().max.y - input_height),
            egui::vec2(avail.x, input_height),
        );
        let mut input_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(input_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        input_ui.add_space(8.0);

        // "+" attachment button (stubbed until we wire real attachments)
        input_ui
            .add_enabled(false, egui::Button::new("➕").min_size(egui::vec2(32.0, 32.0)))
            .on_disabled_hover_text("Attachments coming soon");

        input_ui.add_space(4.0);

        let send_enabled = !has_pending && !state.input.trim().is_empty();
        let text_edit = egui::TextEdit::multiline(&mut state.input)
            .desired_rows(2)
            .desired_width(input_ui.available_width() - 90.0)
            .hint_text(if has_pending {
                "Waiting for response…"
            } else {
                "Type a message and press Enter to send"
            });

        let response = input_ui.add_enabled(!has_pending, text_edit);
        let enter_pressed = response.has_focus()
            && input_ui
                .input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);

        input_ui.add_space(4.0);
        let send_clicked = input_ui
            .add_enabled(
                send_enabled,
                egui::Button::new("Send").min_size(egui::vec2(64.0, 32.0)),
            )
            .clicked();

        if (send_clicked || enter_pressed) && send_enabled {
            send_request = true;
        }
    });

    if send_request {
        app.send_message();
    }

    // Keep repainting while a response is pending or while a message is fading in.
    if has_pending || has_fading_message {
        ctx.request_repaint_after(Duration::from_millis(16));
    }
}

fn render_messages(ui: &mut egui::Ui, messages: Option<&[Message]>) {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.add_space(8.0);

            let empty = match messages {
                None => true,
                Some(m) => m.is_empty(),
            };
            if empty {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Start a conversation below.")
                            .color(egui::Color32::from_gray(140)),
                    );
                });
                return;
            }

            for msg in messages.unwrap() {
                render_message(ui, msg);
                ui.add_space(8.0);
            }
        });
}

fn render_message(ui: &mut egui::Ui, msg: &Message) {
    let is_user = matches!(msg.role, Role::User);

    // Fade-in alpha: only assistant messages fade; user messages are instant.
    let alpha = if is_user {
        255u8
    } else {
        let t = msg.appeared_at.elapsed().as_secs_f32() / FADE_DURATION.as_secs_f32();
        (t.clamp(0.0, 1.0) * 255.0) as u8
    };

    let bubble_color = if is_user {
        egui::Color32::from_rgb(50, 90, 160)
    } else {
        egui::Color32::from_rgb(48, 48, 52)
    };
    let bubble_color = with_alpha(bubble_color, alpha);
    let text_color = with_alpha(egui::Color32::WHITE, alpha);

    let layout = if is_user {
        egui::Layout::right_to_left(egui::Align::Min)
    } else {
        egui::Layout::left_to_right(egui::Align::Min)
    };

    ui.with_layout(layout, |ui| {
        let max_width = ui.available_width() * 0.75;
        egui::Frame::group(ui.style())
            .fill(bubble_color)
            .rounding(egui::Rounding::same(10.0))
            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.set_max_width(max_width);
                ui.label(egui::RichText::new(&msg.content).color(text_color));
            });
    });
}

fn with_alpha(c: egui::Color32, a: u8) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max).collect();
        out.push('…');
        out
    }
}