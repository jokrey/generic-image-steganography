use jokrey_utilities::general::{BitIterator, BytesBuilder, distance, Pushable, StackSlice, Popable};

use crate::difference_encoder::bits_difference_converter::{dynamic_bits_to_difference_if_allowed, get_min_num_bits_encodable, get_num_bits_decodable, dynamic_difference_to_bits, get_max_num_bits_encodable};
use crate::util::{DifCodeImage, DifCodeResult, DifCodeError, EncodingContainer};

pub fn encode_into_vec(message_bytes: &[u8], original: &dyn EncodingContainer, allowed_changes_map: &dyn EncodingContainer) -> DifCodeResult<Vec<u8>> {
    let mut encoded= vec![0; original.len()];
    encode(message_bytes, original, allowed_changes_map, &mut encoded)?;
    Ok(encoded)
}
pub fn encode_into_image(message_bytes: &[u8], original: &DifCodeImage, allowed_changes_map: &dyn EncodingContainer) -> DifCodeResult<DifCodeImage> {
    let mut encoded_image = DifCodeImage::with_capacity(original.width(), original.height());
    encode(&message_bytes, original, allowed_changes_map, &mut encoded_image)?;
    Ok(encoded_image)
}
pub fn encode_into_image_into_path(message_bytes: &[u8], original: &DifCodeImage, allowed_changes_map: &dyn EncodingContainer, path: &str) -> DifCodeResult<()> {
    let encoded_image= encode_into_image(message_bytes, original, allowed_changes_map)?;
    encoded_image.save(path)?;
    Ok(())
}

/// Encodes each bit in the message_bytes slice into the encoded container,
///    making at most a change to the value defined in the allowed_changes_map(and only in that direction).
/// All three containers are required to have the same size.
/// Encoded's content will be fully overridden.
///
///
pub fn encode(message_bytes: &[u8], original: &dyn EncodingContainer, allowed_changes_map: &dyn EncodingContainer, encoded: &mut dyn EncodingContainer) -> DifCodeResult<()> {
    if original.len() != encoded.len() || original.len() != allowed_changes_map.len() {
        return Err(DifCodeError::InternalMismatchedContainerSizes)
    }

    let mut message_bit_iterator = BitIterator::new(message_bytes);

    let mut arr_buf_8_bool_1 = [false; 8];
    let mut bit_buffer = StackSlice::new(&mut arr_buf_8_bool_1);

    for i in 0..original.len() {
        let original_value = original[i];
        let allowed_change = allowed_changes_map[i];
        if original_value == allowed_change {
            encoded[i] = original_value;
            continue;
        }
        let max_allowed_change = distance(original_value, allowed_change);
        let num_bits_at_least_encodable = get_min_num_bits_encodable(max_allowed_change);
        let bit_buffer_len_top = bit_buffer.len() as u8;

        //decode the number of bits that will definitely fit into the allowed difference interval
        let num_to_query = if bit_buffer_len_top >= num_bits_at_least_encodable {0} else {num_bits_at_least_encodable - bit_buffer_len_top};
        for _ in 0..num_to_query {
            if let Some(message_bit) = message_bit_iterator.next() {
                bit_buffer.push(message_bit);
            } else {
                break;
            }
        }

        //decode 1 more bit, to test whether it also fits
        if !bit_buffer.capacity_reached() && bit_buffer.len() as u8 <= num_bits_at_least_encodable {
            if let Some(message_bit) = message_bit_iterator.next() {
                bit_buffer.push(message_bit);
                if let Some(encoded_difference) = dynamic_bits_to_difference_if_allowed(i, bit_buffer.as_slice(), max_allowed_change) {
                    // if let Some(encoded_difference) = static_bits_to_difference_if_allowed(bit_buffer.as_slice(), max_allowed_change) {
                    // println!("apply1 - bit_buffer: {:?}", bit_buffer.as_slice());
                    // println!("apply1 - encoded_difference: {:?}", encoded_difference);
                    bit_buffer.clear();//used all bits, so clear the buffer...
                    apply_change(i, encoded, allowed_change, original_value, encoded_difference);
                    continue;
                }
            }
        }
        //else, if bit_buffer capacity reached(never occurs afaik), no more bits in message or encoding 1 more bit is not allowed

        // println!("xx - bit_buffer: {:?}", bit_buffer.as_slice());
        let mut change_applied = false;
        let len_before = bit_buffer.len();
        while !bit_buffer.is_empty() {
            let last_encodable_bits = bit_buffer.as_slice();
            if let Some(encoded_difference) = dynamic_bits_to_difference_if_allowed(i, last_encodable_bits, max_allowed_change) {
                // if let Some(encoded_difference) = static_bits_to_difference_if_allowed(last_encodable_bits, max_allowed_change) {
                // println!("apply2 - bit_buffer: {:?}", bit_buffer.as_slice());
                // println!("apply2 - encoded_difference: {:?}", encoded_difference);
                apply_change(i, encoded, allowed_change, original_value, encoded_difference);
                change_applied = true;
                break;
            }
            bit_buffer.delete_top();
        }
        unsafe {
            let len_now = bit_buffer.len();
            bit_buffer.set_len(len_before);
            bit_buffer.clear_range(0, len_now); //clear those currently remaining in buffer
        }
        // println!("yy - bit_buffer: {:?}", bit_buffer.as_slice());
        if ! change_applied {
            encoded[i] = original_value;
        }
    }

    if !bit_buffer.is_empty() || message_bit_iterator.num_remaining() > 0 {
        // println!("bit_buffer = {:?}", bit_buffer.as_slice());
        return Err(DifCodeError::InternalCapacityReached(message_bytes.len()*8 - (message_bit_iterator.num_remaining() + bit_buffer.len())));
    }



    Ok(())
}

fn apply_change(index: usize, encoded: &mut dyn EncodingContainer,
                allowed_change: u8, original_value: u8, target_difference: u8) {
    if allowed_change < original_value {
        encoded[index] = original_value - target_difference
    } else {
        encoded[index] = original_value + target_difference;
    }
}


pub fn decode_into_vec(original: &dyn EncodingContainer, encoded: &dyn EncodingContainer) -> DifCodeResult<Vec<u8>> {
    if original.len() != encoded.len() {
        return Err(DifCodeError::InternalMismatchedContainerSizes)
    }

    let mut decoded_message = Vec::with_capacity(get_encoded_message_length_in_bits(original, encoded));
    decode(original, encoded, &mut decoded_message)?;
    Ok(decoded_message)
}

pub fn decode(original: &dyn EncodingContainer, encoded: &dyn EncodingContainer, message_buffer: &mut dyn Pushable<u8>) -> DifCodeResult<()> {
    if original.len() != encoded.len() {
        return Err(DifCodeError::InternalMismatchedContainerSizes)
    }

    let mut message_builder = BytesBuilder::new(message_buffer);

    let mut bit_buffer = [false; 8];
    let mut num_pushed_counter = 0;
    for i in 0..original.len() {
        let difference = distance(original[i], encoded[i]);
        // println!("original[i]: {}", original[i]);
        // println!("encoded[i]: {}", encoded[i]);
        // println!("difference: {}", difference);
        let num_bits_decodable = get_num_bits_decodable(difference) as usize;
        // println!("num_bits_decodable: {}", num_bits_decodable);
        dynamic_difference_to_bits(i, difference, &mut bit_buffer[0..num_bits_decodable]);
        // println!("bit_buffer[0..num_bits_decodable]: {:?}", &bit_buffer[0..num_bits_decodable]);
        // static_difference_to_bits(difference, &mut bit_buffer[0..num_bits_decodable]);
        for j in 0..num_bits_decodable {
            if !message_builder.push(bit_buffer[j]) {
                return Err(DifCodeError::Internal("could not push bit"))
            }
            num_pushed_counter+=1;
        }
    }

    // println!("num pushed: {}", num_pushed_counter);

    Ok(())
}



pub fn get_encoded_message_length_in_bits(original: &dyn EncodingContainer, encoded: &dyn EncodingContainer) -> usize {
    if original.len() != encoded.len() {
        panic!("original len != allowed_changes_map len");
    }

    let mut counter: usize = 0;
    for i in 0..original.len() {
        counter += get_num_bits_decodable(distance(original[i], encoded[i])) as usize;
    }
    counter
}
pub fn get_max_encodable_message_length_in_bits(original: &dyn EncodingContainer, allowed_changes_map: &dyn EncodingContainer) -> usize {
    if original.len() != allowed_changes_map.len() {
        panic!("original len != allowed_changes_map len");
    }

    let mut counter: usize = 0;
    for i in 0..original.len() {
        counter += get_max_num_bits_encodable(distance(original[i], allowed_changes_map[i])) as usize;
    }
    counter
}
pub fn get_min_encodable_message_length_in_bits(original: &dyn EncodingContainer, allowed_changes_map: &dyn EncodingContainer) -> usize {
    if original.len() != allowed_changes_map.len() {
        panic!("original len != allowed_changes_map len");
    }

    let mut counter: usize = 0;
    for i in 0..original.len() {
        counter += get_min_num_bits_encodable(distance(original[i], allowed_changes_map[i])) as usize;
    }
    counter
}