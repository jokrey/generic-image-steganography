use rand::{Rng, thread_rng};

use jokrey_utilities::general::{get_bit_at, sample_unique, set_bit_at};

use crate::util::{DifCodeError, DifCodeImage, DifCodeResult, EncodingContainer, get_length_in_bits};

pub fn randomly_select_indices_within(message_bytes: &[u8], original: &dyn EncodingContainer) -> Vec<usize> {
    randomly_select_indices(get_length_in_bits(message_bytes), original.len())
}
pub fn randomly_select_indices(message_length_in_bits: usize, max_index: usize) -> Vec<usize> {
    let mut selected_indices = sample_unique(0..max_index, message_length_in_bits);
    selected_indices.sort();
    selected_indices
}

pub fn encode_into_vec_at_indices(message_bytes: &[u8], original: &dyn EncodingContainer, selected_indices: &[usize]) -> DifCodeResult<Vec<u8>> {
    let mut encoded= vec![0; original.len()];
    encode_at_indices(message_bytes, original, selected_indices, &mut encoded)?;
    Ok(encoded)
}
pub fn encode_into_image_at_indices(message_bytes: &[u8], original: &DifCodeImage, selected_indices: &[usize]) -> DifCodeResult<DifCodeImage> {
    let mut encoded_image = DifCodeImage::with_capacity(original.width(), original.height());
    encode_at_indices(&message_bytes, original, selected_indices, &mut encoded_image)?;
    Ok(encoded_image)
}
pub fn encode_into_image_into_path_at_indices(message_bytes: &[u8], original: &DifCodeImage, selected_indices: &[usize], path: &str) -> DifCodeResult<()> {
    let encoded_image= encode_into_image_at_indices(message_bytes, original, selected_indices)?;
    encoded_image.save(path)?;
    Ok(())
}

///Note: - selected_indices must be sorted in ascending order and each entry must represent an index within the original container
pub fn encode_at_indices(message_bytes: &[u8], original: &dyn EncodingContainer, selected_indices: &[usize], encoded: &mut dyn EncodingContainer) -> DifCodeResult<()> {
    if get_length_in_bits(message_bytes) < selected_indices.len() {
        Err(DifCodeError::from("message length in bits > selected indices length (too many selected indices, ambiguity must be resolved by caller)"))
    } else if get_length_in_bits(message_bytes) > selected_indices.len() { //here the longer code tables come into play
        Err(DifCodeError::from("message length in bits > selected indices length (too few selected indices)"))
    } else {
        let mut rng = thread_rng();

        let mut index_in_original = 0;
        let mut selected_indices_iterator = selected_indices.iter();
        for message_byte in message_bytes {
            for bit_i in 0..8 {
                let bit_to_encode = get_bit_at(*message_byte, bit_i);

                let selected_index_to_encode_at = selected_indices_iterator.next().unwrap();
                while index_in_original < *selected_index_to_encode_at {
                    encoded[index_in_original] = original[index_in_original];
                    index_in_original += 1;
                }

                let goal_difference: u8 = if !bit_to_encode { 1 } else { 2 };
                encode_difference_into(*selected_index_to_encode_at, original, encoded, goal_difference, rng.gen_bool(0.5));
                index_in_original += 1;
            }
        }
        while index_in_original < original.len() {
            encoded[index_in_original] = original[index_in_original];
            index_in_original += 1;
        }
        Ok(())
    }
}

fn encode_difference_into(index: usize, original: &dyn EncodingContainer, encoded: &mut dyn EncodingContainer, goal_difference: u8, difference_sign_positive_desired: bool) {
    let original_value = original[index];
    if difference_sign_positive_desired {
        if original_value > 255 - goal_difference { //does not fit(would go over 255), must subtract - despite wishes
            encoded[index] = original_value - goal_difference;
        } else {
            encoded[index] = original_value + goal_difference;
        }
    } else {
        if original_value < goal_difference { //does not fit(would go under 0), must add - despite wishes
            encoded[index] = original_value + goal_difference;
        } else {
            encoded[index] = original_value - goal_difference;
        }
    }
}






pub fn decode_into_vec_at_indices(original: &dyn EncodingContainer, encoded: &dyn EncodingContainer) -> DifCodeResult<Vec<u8>> {
    if original.len() != encoded.len() {
        panic!("original data length != encoded data length")
    }

    let msg_length_in_bits = get_single_bit_encoded_message_length_in_bits(original, encoded);
    if ! (msg_length_in_bits % 8 == 0) {
        return Err(DifCodeError::from("invalid input data (message length incorrect)"));
    }

    let mut decoded_message = vec![0u8; msg_length_in_bits/8];

    decode_at_indices(original, encoded, msg_length_in_bits, decoded_message.as_mut_slice())?;

    Ok(decoded_message)
}

pub fn decode_at_indices(original: &dyn EncodingContainer, encoded: &dyn EncodingContainer, msg_length_in_bits: usize, decoded_message_buffer: &mut [u8]) -> DifCodeResult<()> {
    let mut code_text_index:usize = 0;
    let mut message_index = 0;
    let mut message_byte = 0u8;
    let mut message_bit_index = 0;
    for _ in 0..msg_length_in_bits {
        let mut decoded_bit = None;
        while decoded_bit == None && code_text_index < original.len() {
            decoded_bit = decode_from_difference_at_indices(code_text_index, original, encoded);
            code_text_index += 1;
        }

        if decoded_bit == Some(true) {
            message_byte = set_bit_at(message_byte, message_bit_index);
        }

        message_bit_index += 1;
        if message_bit_index == 8 {
            decoded_message_buffer[message_index] = message_byte;

            message_index += 1;
            message_byte = 0u8;
            message_bit_index = 0;
        }
    }
    Ok(())
}


fn decode_from_difference_at_indices(index: usize, original: &dyn EncodingContainer, encoded: &dyn EncodingContainer) -> Option<bool> {
    let difference = original[index] as isize - encoded[index] as isize;

    match difference.abs() {
        1 => Some(false),
        2 => Some(true),
        _ => None
    }
}

pub fn get_single_bit_encoded_message_length_in_bits(original: &dyn EncodingContainer, encoded: &dyn EncodingContainer) -> usize {
    let mut counter: usize = 0;
    for index in 0..original.len() {
        if original[index] != encoded[index] {
            counter += 1;
        }
    }
    return counter;
}
