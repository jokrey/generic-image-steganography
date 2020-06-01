extern crate base64;
extern crate image;
extern crate rand;
extern crate reqwest;

use difference_encoder::{*};
use jokrey_utilities::general::Wrapper;
use jokrey_utilities::tui_menu_interface::{Choice, ChoiceConstrainedInput, InputItem, Menu, NonExistingPathInput};
use std::ops::Deref;
use jokrey_utilities::encoding::tag_based::bytes::remote::authenticated::authentication_helper::{aes_crt_np_128_encrypt_into_decipherable, aes_crt_np_128_decrypt_from_decipherable};

mod difference_encoder;

fn main() {
    let encode_or_decode_choice = Choice::standalone("Encode or Decode?", vec!["Encode", "Decode"]);

    match encode_or_decode_choice.run_for_value().as_deref() {
        Some("Encode") => encode_menu(),
        Some("Decode") => decode_menu(),
        Some(_) | None => println!("Successfully canceled, thanks for choosing image steganography difference coding system.")
    }
}



fn encode_menu() {
    let message_chooser = ChoiceConstrainedInput::new("Message", vec!["UTF8", "Decode from Base64"], |raw, choice| {
        match choice {
            "UTF8" => { Ok(Wrapper::from(raw.as_bytes().to_vec())) },
            "Decode from Base64" => {
                match base64::decode(raw) {
                    Ok(decoded) => Ok(Wrapper::from(decoded)),
                    Err(_) => Err("could not base64 decode")
                }
            },
            _ => Err("Unknown Encoding (impossible)")
        }
    });
    let encryption_chooser = ChoiceConstrainedInput::new("Encryption: ", vec!["AES"], |raw, _| Ok(raw.to_string()));
    let image_chooser = new_image_chooser("Load Original Image");
    let output_path_chooser = NonExistingPathInput::new_nep("Output Image Path");

    Menu::run_root("Encrypt Your Message Into Your Images", vec![
        &message_chooser,
        &encryption_chooser,
        &image_chooser,
        &output_path_chooser
    ]);

    let message = message_chooser.get_value();
    if let Some(message) = message {
        let encryption = encryption_chooser.get_value();
        let final_message_bytes = match encryption {
            None => {message.raw().1.raw()}
            Some(encryption) => {
                match encryption.get_0().deref() {
                    "AES" => {
                        let pw_as_string = encryption.get_1();
                        aes_crt_np_128_encrypt_into_decipherable(&message.raw().1.raw(), pw_as_string)
                    }
                    _ => message.raw().1.raw() //unknown encryption choice, impossible
                }
            }
        };

        let image = image_chooser.get_value();
        if let Some(image) = image {
            let image = image.get_1();
            let output_path = output_path_chooser.get_value();
            if let Some(output_path) = output_path {
                println!("Encoding final message({:?}),\n    into image({}),\n    and storing in path:\n{}", &final_message_bytes, &image, &output_path);
                encode_into_image_into_path(&final_message_bytes, image, &randomly_select_indices_within(&final_message_bytes, image), &output_path).expect("failed to encode");
            } else {
                println!("Missing image - cannot encode message into no image")
            }
        } else {
            println!("Missing image - cannot encode message into no image")
        }
    } else {
        println!("Missing message - cannot encode no message")
    }
}



fn decode_menu() {
    let image1_chooser = new_image_chooser("Load Original/Encoded Image");
    let image2_chooser = new_image_chooser("Load Encoded/Original Image");
    let encryption_chooser = ChoiceConstrainedInput::new("Decryption: ", vec!["AES"], |raw, _| Ok(raw.to_string()));
    let decoding_chooser = Choice::new("Decoding: ", vec!["UTF8", "Base64"]);

    Menu::run_root("Decrypt Your Message From Images", vec![
        &image1_chooser,
        &image2_chooser,
        &encryption_chooser,
        &decoding_chooser
    ]);


    let image1 = image1_chooser.get_value();
    if let Some(image1) = image1 {
        let image1 = image1.get_1();
        let image2 = image2_chooser.get_value();
        if let Some(image2) = image2 {
            let image2 = image2.get_1();

            let decoded_raw_bytes = decode_into_vec(image2, image1); //order irrelevant

            match decoded_raw_bytes {
                Ok(decoded_raw_bytes) => {
                    let encryption = encryption_chooser.get_value();
                    let final_message_bytes = match encryption {
                        None => { decoded_raw_bytes }
                        Some(encryption) => {
                            match encryption.get_0().deref() {
                                "AES" => {
                                    let pw_as_string = encryption.get_1();
                                    aes_crt_np_128_decrypt_from_decipherable(&decoded_raw_bytes, pw_as_string)
                                }
                                _ => decoded_raw_bytes //unknown encryption choice, impossible
                            }
                        }
                    };

                    match decoding_chooser.get_value().as_deref() {
                        Some("UTF8") => match std::str::from_utf8(&final_message_bytes) {
                            Ok(utf8) => println!("MESSAGE (utf8 decoded): \n{}", utf8),
                            Err(_) => println!("MESSAGE (raw, COULD NOT BE UTF8 DECODED: {:?}", final_message_bytes)
                        },
                        Some("Base64") => println!("MESSAGE (base64 encoded): \n{}", base64::encode(&final_message_bytes)),
                        Some(_) | None => println!("Invalid Decoding selected (perhaps forgotten?)")
                    };
                }
                Err(err) => {
                    println!("Decoding failed({:?}).\nAre you sure the images are dif decodable?", err)
                }
            }
        } else {
            println!("Missing an image - cannot decode message without 2 'identical' images")
        }
    } else {
        println!("Missing an image - cannot decode message without 2 'identical' images")
    }
}



fn new_image_chooser(name: &str) -> ChoiceConstrainedInput<DifCodeImage> {
    ChoiceConstrainedInput::new(name, vec!["URL", "Path"], |raw, choice| {
        match choice {
            "Path" => {
                DifCodeImage::open(raw).map_err(|_| "Failed to load image from path")
            }
            "URL" => {
                let img_bytes = reqwest::blocking::get(raw).map_err(|_| "Failed to download image from url")?.bytes().map_err(|_| "Failed to convert downloaded image to bytes")?;

                DifCodeImage::from_memory(&img_bytes).map_err(|_| "Failed to load image from path")
            }
            _ => Err("Invalid Input for Choice (impossible)")
        }
    })
}