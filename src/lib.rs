#![allow(clippy::wildcard_imports)]

use seed::{prelude::*, *};

// use some constants for pictures
// wasm saved on github has /hangman-seed/hm/0.png to work around github pages' sub-directory
// (same in index.html)
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
const WON_GAME_IMAGE: &str = "/hangman-seed/hm/win.png";

// ------ ------
//     Init
// ------ ------
fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    Model {
        show_secret_cleartext: false,
        secret_string: vec![],
        displayed_secret: vec![],
        guessed_letters: vec![],
        incorrect_guessed_letters: vec![],
        game_started: false,
        last_found_number: None,
        event_streams: vec![],
    }
}

// ------ ------
//     Model
// ------ ------
#[derive(PartialEq)]
struct SecretLetter {
    letter: char,
    displayed: bool,
}

struct Model {
    show_secret_cleartext: bool,
    secret_string: Vec<SecretLetter>,
    displayed_secret: Vec<Node<Msg>>,
    guessed_letters: Vec<char>,
    incorrect_guessed_letters: Vec<char>,
    game_started: bool,
    last_found_number: Option<u32>,

    // this is to listen to the keyboard during the game
    event_streams: Vec<StreamHandle>,
}

impl Model {
    fn start_new_game(&mut self) {
        self.game_started = false;
        self.show_secret_cleartext = false;
        self.incorrect_guessed_letters = vec![];
        self.guessed_letters = vec![];
        self.displayed_secret = vec![];
        self.secret_string = vec![];

        // stop listening to the keyboard
        self.event_streams.clear();
    }
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
            model.start_new_game();
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

            if &key_code == &(27 as u32) {
                // not guessing a letter, pushing escape will start a new game, so clear then return
                model.start_new_game();
                return;
            }

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

            // keep track of how many letters the last guess connected with
            let mut last_found_number: u32 = 0;

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
                    if matched_letter {
                        letter_was_a_match = true;
                        last_found_number += 1;
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
                model.last_found_number = Some(last_found_number);
            } else {
                model.incorrect_guessed_letters.push(uppercase);
                model.last_found_number = None;
            }
        }
    }
}

// ------ ------
//     View
// ------ ------
fn view(model: &Model) -> Node<Msg> {
    let display_secret_or_plaintext = if model.show_secret_cleartext {
        "text"
    } else {
        "password"
    };

    let not_there_list = &model.incorrect_guessed_letters;

    let not_there_list: String = format!("no: {}", not_there_list.iter().collect::<String>());

    // gets the image to display (0 wrong = pic[0]... 5 wrong = pic[5])
    let pic_link = if model.secret_string.iter().any(|letter| !letter.displayed) {
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
                br!(),
                &model.displayed_secret,
                br!(),br!(),
                print_last_found_number(model),
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

fn print_last_found_number(model: &Model) -> Node<Msg> {
    if let Some(last_found) = model.last_found_number {
        div![format!("Found: {}", last_found)]
    } else {
        div![]
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
fn convert_secret_char_list_to_real_string(letter_list: &[SecretLetter]) -> Vec<Node<Msg>> {
    let mut new_displayed_secret_string: Vec<char> = vec![];
    for letter in letter_list {
        if letter.displayed {
            new_displayed_secret_string.push(letter.letter);
        } else {
            new_displayed_secret_string.push('_');
        }
    }

    new_displayed_secret_string.into_iter().collect::<String>()
        .trim()
        .split(' ')
        .filter_map(|word| {
            // filter out any accidental double spaces
            if word.len() > 0 {
                Some(span!(word, C!["displayed_word large_letters"]))
            } else {
                None
            }
        })
        .collect()
}
