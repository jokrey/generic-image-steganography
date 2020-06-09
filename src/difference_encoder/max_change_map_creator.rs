use crate::util::{get_length_in_bits, EncodingContainer};
use crate::difference_encoder::bits_difference_converter::{get_max_num_bits_encodable, calculate_worst_case_difference_for};
use jokrey_utilities::general::distance;
use crate::rand::prelude::SliceRandom;

pub fn create_minimal_random_allowed_changes_map_for(message: &[u8], original: &dyn EncodingContainer, max_difference: u8) -> Vec<u8> {
    let mut output_map = vec![0; original.len()];
    write_minimal_random_allowed_changes_map_with(get_length_in_bits(message), original, max_difference, &mut output_map);
    output_map
}
pub fn create_minimal_random_allowed_changes_map(message: &[u8], original: &dyn EncodingContainer) -> Vec<u8> {
    let mut output_map = vec![0; original.len()];
    write_minimal_random_allowed_changes_map_for(message, original, &mut output_map);
    output_map
}
pub fn write_minimal_random_allowed_changes_map_for(message: &[u8], original: &dyn EncodingContainer, output_map: &mut dyn EncodingContainer) {
    write_minimal_random_allowed_changes_map_with(get_length_in_bits(message), original, 255, output_map)
}
pub fn write_minimal_random_allowed_changes_map_with(mut message_length_in_bits: usize, original: &dyn EncodingContainer, max_difference: u8, output_map: &mut dyn EncodingContainer) {
    for i in 0..original.len() {
        output_map[i] = original[i];
    }

    let mut all_indices: Vec<usize> = (0..original.len()).collect();
    let mut rng = rand::thread_rng();

    while message_length_in_bits > 0 {
        all_indices.shuffle(&mut rng);
        let selected_indices = &all_indices[0..message_length_in_bits.min(all_indices.len())];

        let mut success_counter = 0;
        for sel_i in selected_indices {
            let original_value = original[*sel_i];
            let allowed_change = output_map[*sel_i];
            let old_difference = distance(original_value, allowed_change);

            if old_difference < max_difference {
                let num_bits_currently_encodable = get_max_num_bits_encodable(old_difference);
                if let Some(mut new_difference) = calculate_worst_case_difference_for(num_bits_currently_encodable + 1) {
                    if new_difference > max_difference {
                        new_difference = max_difference;
                    }
                    if original_value == allowed_change {
                        let is_direction_positive = original_value <= 255 / 2; //maximize possibility
                        if attempt_change(output_map, *sel_i, new_difference - old_difference, is_direction_positive) {
                            success_counter += 1;
                        }
                    } else {
                        let desired_direction_positive = allowed_change > original_value;//keep same direction as before...
                        if attempt_change(output_map, *sel_i, new_difference - old_difference, desired_direction_positive) {
                            success_counter += 1;
                        }
                    }
                }
            }
        }

        if success_counter == 0 {
            panic!("Cannot encode message - not enough space in encoding container")
        }

        message_length_in_bits -= success_counter;
    }
}



fn attempt_change(container: &mut dyn EncodingContainer, index: usize, desired_distance: u8, desired_direction_positive: bool) -> bool {
    let original_value = container[index];
    if desired_direction_positive {
        if original_value > 255 - desired_distance {
            false
        } else {
            container[index] = original_value + desired_distance;
            true
        }
    } else {
        if original_value < desired_distance {
            false
        } else {
            container[index] = original_value - desired_distance;
            true
        }
    }
}