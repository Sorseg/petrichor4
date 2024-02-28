use bevy::app::{App, Plugin, Update};
use bevy::asset::AssetServer;
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::{AlignItems, BackgroundColor, BuildChildren, Camera2dBundle, Color, Commands, Component, default, Entity, EventReader, FlexDirection, in_state, IntoSystemConfigs, JustifyContent, JustifyText, KeyCode, NextState, NodeBundle, OnEnter, OnExit, Outline, Query, ReceivedCharacter, Res, ResMut, Resource, Style, Text, TextBundle, TextStyle, Val, With};
use crate::plugin::{PetriState};

pub struct LoginPlugin;

#[derive(Resource, Debug)]
pub struct CurrentUserLogin(pub String);

#[derive(Debug, Component)]
struct LoginInput;

#[derive(Debug, Component)]
struct LoginUIMarker;

impl Plugin for LoginPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(CurrentUserLogin(String::new()))
            .add_systems(OnEnter(PetriState::Login), login_screen)
            .add_systems(Update, login_input.run_if(in_state(PetriState::Login)))
            .add_systems(
                OnExit(PetriState::Login),
                |mut cmd: Commands, login_entity: Query<Entity, With<LoginUIMarker>>| {
                    login_entity
                        .iter()
                        .for_each(|l| cmd.entity(l).despawn_recursive())
                },
            )
        ;
    }
}

/// crete ui entities for login screen
fn login_screen(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn((Camera2dBundle::default(), LoginUIMarker));
    let root = cmd
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: BackgroundColor(Color::DARK_GRAY),
                ..default()
            },
            LoginUIMarker,
        ))
        .id();

    let prompt = cmd
        .spawn(
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "Login",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font: asset_server.load("open-sans.ttf"),
                    font_size: 100.0,
                    ..default()
                },
            ) // Set the alignment of the Text
                .with_text_justify(JustifyText::Center),
        )
        .id();

    let login_container = cmd
        .spawn((
            NodeBundle {
                background_color: BackgroundColor(Color::DARK_GREEN),
                ..default()
            },
            Outline {
                width: Val::Px(6.),
                offset: Val::Px(6.),
                color: Color::LIME_GREEN,
            },
        ))
        .id();

    let input = cmd
        .spawn((
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "_",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font: asset_server.load("open-sans.ttf"),
                    font_size: 100.0,
                    ..default()
                },
            ) // Set the alignment of the Text
                .with_text_justify(JustifyText::Center),
            LoginInput,
        ))
        .id();

    cmd.entity(login_container).add_child(input);
    cmd.entity(root)
        .add_child(prompt)
        .add_child(login_container);
}

/// receive login credentials into the login field
fn login_input(
    mut char_input_events: EventReader<ReceivedCharacter>,
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut login_label: Query<&mut Text, With<LoginInput>>,
    mut login: ResMut<CurrentUserLogin>,
    mut next_state: ResMut<NextState<PetriState>>,
) {
    for event in keyboard_input_events.read() {
        match event.key_code {
            KeyCode::Enter => {
                if !login.0.trim().is_empty() {
                    next_state.set(PetriState::Scene);
                    return;
                }
            }
            KeyCode::Backspace => {
                login.0.pop();
            }
            _ => {}
        }
    }
    for event in char_input_events.read() {
        login.0.push_str(event.char.as_str());
    }
    login_label.iter_mut().for_each(|mut l| {
        l.sections[0].value = format!("{}_", login.0);
    })
}

