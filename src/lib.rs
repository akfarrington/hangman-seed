#![allow(clippy::wildcard_imports)]

use seed::{prelude::*, *};

// use some constants for pictures
const GAME_IMAGES: [&str; 11] = [
    "/hangman-seed/hm/0.png",
    "/hangman-seed/hm/1.png",
    "/hangman-seed/hm/2.png",
    "/hangman-seed/hm/3.png",
    "/hangman-seed/hm/4.png",
    "/hangman-seed/hm/5.png",
    "/hangman-seed/hm/6.png",
    "/hangman-seed/hm/7.png",
    "/hangman-seed/hm/8.png",
    "/hangman-seed/hm/9.png",
    "/hangman-seed/hm/10.png",
];
const WON_GAME_IMAGE: &str = "/hm/win.png";

// ------ ------
//     Init
// ------ ------
fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    Model {
        show_secret_cleartext: false,
        secret_string: vec![],
        displayed_secret: "".to_string(),
        guessed_letters: vec![],
        incorrect_guessed_letters: vec![],
        game_started: false,
        event_streams: vec![],
    }
}

// ------ ------
//     Model
// ------ ------
struct SecretLetter {
    letter: char,
    displayed: bool,
}

struct Model {
    show_secret_cleartext: bool,
    secret_string: Vec<SecretLetter>,
    displayed_secret: String,
    guessed_letters: Vec<char>,
    incorrect_guessed_letters: Vec<char>,
    game_started: bool,

    // this is to listen to the keyboard during the game
    event_streams: Vec<StreamHandle>,
}

// ------ ------
//    Update
// ------ ------
enum Msg {
    ToggleDisplayHideSecret,
    ClearGame,
    NewSecretFieldChanged(String),
    StartGame,
    GuessLetter(web_sys::KeyboardEvent),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        // toggles password field or cleartext when entering new secret for game
        Msg::ToggleDisplayHideSecret => {
            model.show_secret_cleartext = !model.show_secret_cleartext;
        }

        // ends the game and clears the model for the next time
        Msg::ClearGame => {
            model.game_started = false;
            model.show_secret_cleartext = false;
            model.incorrect_guessed_letters = vec![];
            model.guessed_letters = vec![];
            model.displayed_secret = "".to_string();
            model.secret_string = vec![];

            // stop listening to the keyboard
            model.event_streams.clear();
        }

        // while typing a new secret, update in real-time
        Msg::NewSecretFieldChanged(secret) => {
            let mut new_secret_string = vec![];
            for letter in secret.chars() {
                new_secret_string.push(SecretLetter {
                    letter,
                    displayed: !letter.is_ascii_alphabetic(),
                });
            }
            model.secret_string = new_secret_string;
        }

        // starts the game (should show the game instead of the new secret field
        Msg::StartGame => {
            // don't do anything if the secret is too short
            if model.secret_string.len() <= 1 {
                return;
            }

            // start listening to the keyboard (from https://github.com/seed-rs/seed/blob/master/examples/window_events/src/lib.rs)
            if model.event_streams.is_empty() {
                model.event_streams = vec![orders
                    .stream_with_handle(streams::window_event(Ev::KeyDown, |event| {
                        Msg::GuessLetter(event.unchecked_into())
                    }))];
            }

            // start the game, populate the displayed secret string
            // save this in the model
            model.displayed_secret = convert_secret_char_list_to_real_string(&model.secret_string);

            // change the model to indicate the game has started
            model.game_started = true;
        }

        // processes the guessing of a letter
        Msg::GuessLetter(event) => {
            let key_code: u32 = event.key_code();

            // if outside of these bounds it's not a regular letter, so just return
            if !(65..=90).contains(&key_code) {
                return;
            }

            // now convert the keycode to letter. It's always uppercase, then get lowercase
            let uppercase = char::from_u32(key_code).unwrap();
            let lowercase = uppercase.to_ascii_lowercase();

            // check if this letter was guessed, and if so, just quit this part
            if model.guessed_letters.contains(&uppercase) {
                return;
            }

            // keep track of whether this letter was used
            let mut letter_was_a_match = false;

            // copy the model secret string Vec and set the display to true if the letters match
            let new_secret_string_list: Vec<SecretLetter> = model
                .secret_string
                .iter()
                .map(|letter| {
                    // check if the letter matches
                    let matched_letter = letter.letter == uppercase || letter.letter == lowercase;
                    // true if already displayed, or the guessed letter matches the iterated letter
                    let displayed = letter.displayed || matched_letter;

                    // if we found a matched letter, update the letter_was_a_match so I know whether
                    // to add this guessed letter to the failed guesses or not
                    if !letter_was_a_match && matched_letter {
                        letter_was_a_match = true;
                    }

                    SecretLetter {
                        displayed,
                        letter: letter.letter,
                    }
                })
                .collect();

            // add this guessed letter to the guessed letters list
            model.guessed_letters.push(uppercase);

            // now, if there was a matched letter, update the model accordingly, if not, update the failed guesses
            if letter_was_a_match {
                model.displayed_secret =
                    convert_secret_char_list_to_real_string(&new_secret_string_list);
                model.secret_string = new_secret_string_list;
            } else {
                model.incorrect_guessed_letters.push(uppercase);
            }
        }
    }
}

// ------ ------
//     View
// ------ ------
fn view(model: &Model) -> Node<Msg> {
    let display_secret_or_plaintext = if model.show_secret_cleartext {
        "password"
    } else {
        "text"
    };

    let not_there_list = &model.incorrect_guessed_letters;

    let not_there_list: String = format!("no: {}", not_there_list.iter().collect::<String>());

    // gets the image to display (0 wrong = pic[0]... 5 wrong = pic[5])
    let pic_link = if model.displayed_secret.contains('_') {
        let mut number_wrong = model.incorrect_guessed_letters.len();
        if number_wrong >= GAME_IMAGES.len() {
            number_wrong = GAME_IMAGES.len() - 1;
        }
        GAME_IMAGES[number_wrong]
    } else {
        WON_GAME_IMAGE
    };

    if model.game_started {
        div![
            C!["game"],
            div![
                C!["game_image"],
                img!(attrs!(At::Src => pic_link), C!["game_image"]),
                p![not_there_list, C!["large_letters"]],
            ],
            div![
                C!["guesses"],
                p![&model.displayed_secret, C!["large_letters"]],
                br!(),
                br!(),
                button!["New Game!", ev(Ev::Click, move |_| Msg::ClearGame)],
            ]
        ]
    } else {
        div![
            C!["secret_input large_letters"],
            "Enter a new word or sentence:",
            br!(),
            input![
                C!["new_secret_box"],
                input_ev(Ev::Input, Msg::NewSecretFieldChanged),
                keyboard_ev(Ev::KeyDown, |key| {
                    IF!(key.key() == "Enter" => Msg::StartGame)
                }),
                attrs!(
                    At::AutoFocus => "autofocus",
                    At::Type => display_secret_or_plaintext
                )
            ],
            button!["Start!", ev(Ev::Click, move |_| Msg::StartGame)],
            button![
                "show/hide answer!",
                ev(Ev::Click, move |_| Msg::ToggleDisplayHideSecret)
            ]
        ]
    }
}

// ------ ------
//     Start
// ------ ------
#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}

// this is a function to help me change a Vec<SecretLetter> to a string
fn convert_secret_char_list_to_real_string(letter_list: &[SecretLetter]) -> String {
    let mut new_displayed_secret_string: Vec<char> = vec![];
    for letter in letter_list {
        if letter.displayed {
            new_displayed_secret_string.push(letter.letter);
        } else {
            new_displayed_secret_string.push('_');
        }
    }

    new_displayed_secret_string.into_iter().collect()
}

