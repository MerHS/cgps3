use bevy::prelude::*;

pub struct HelloPlugin;

#[derive(Component)]
struct Name(String);

#[derive(Component)]
struct Person;

#[derive(Resource)]
struct GreetTimer(Timer);

fn add_people(mut commands: Commands) {
    commands.spawn((Name("Elaina Proctor".to_string()), Person));
    commands.spawn((Name("Renzo Hume".to_string()), Person));
    commands.spawn((Name("Zayna Nieves".to_string()), Person));
}

fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    // update our timer with the time elapsed since the last update
    // if that caused the timer to finish, we say hello to everyone
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            println!("hello {}!", name.0);
        }
    }
}

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(add_people)
            .insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
            .add_system(greet_people);
    }
}
