use crate::util::{get_length_in_bits, EncodingContainer, DifCodeImage, DifCodeResult, DifCodeError};
use crate::difference_encoder::bits_difference_converter::{calculate_worst_case_difference_for, get_max_num_bits_encodable};
use jokrey_utilities::general::distance;
use crate::rand::prelude::SliceRandom;

//having a max_difference different to the num bits boundaries can cause issues, because the algorithm is conservative. I.e. it ensures that the message can also be encoded in the worst case, not just in the best case.
pub fn create_minimal_evenly_random_allowed_changes_map_for(message: &[u8], original: &dyn EncodingContainer, max_difference: u8) -> DifCodeResult<Vec<u8>> {
    let mut output_map = vec![0; original.len()];
    write_minimal_evenly_random_allowed_changes_map_with(get_length_in_bits(message), original, max_difference, &mut output_map)?;
    Ok(output_map)
}
pub fn create_minimal_evenly_random_allowed_changes_map_for_image(message: &[u8], original: &DifCodeImage) -> DifCodeResult<Vec<u8>> {
    let mut output_map = vec![0; original.len()];
    write_minimal_evenly_random_allowed_changes_map_for(message, original, &mut output_map)?;
    Ok(output_map)
}
pub fn create_minimal_evenly_random_allowed_changes_map(message: &[u8], original: &dyn EncodingContainer) -> DifCodeResult<Vec<u8>> {
    let mut output_map = vec![0; original.len()];
    write_minimal_evenly_random_allowed_changes_map_for(message, original, &mut output_map)?;
    Ok(output_map)
}
pub fn write_minimal_evenly_random_allowed_changes_map_for(message: &[u8], original: &dyn EncodingContainer, output_map: &mut dyn EncodingContainer) -> DifCodeResult<()> {
    write_minimal_evenly_random_allowed_changes_map_with(get_length_in_bits(message), original, 255, output_map)
}

//having a max_difference different to the num bits boundaries can cause issues, because the algorithm is conservative. I.e. it ensures that the message can also be encoded in the worst case, not just in the best case.
pub fn write_minimal_evenly_random_allowed_changes_map_with(message_length_in_bits: usize, original: &dyn EncodingContainer, max_difference: u8, output_map: &mut dyn EncodingContainer) -> DifCodeResult<()> {
    write_minimal_evenly_random_allowed_changes_map(message_length_in_bits, original,
                                move |_index, ov| (max_difference, ov <= 255 / 2), //maximize possibility
                                                   output_map)
}
pub fn write_minimal_evenly_random_allowed_changes_map<F>(message_length_in_bits: usize,
                                                          original: &dyn EncodingContainer,
                                                          change_constraint_calculator: F,
                                                          output_map: &mut dyn EncodingContainer) -> DifCodeResult<()>
where F: Fn(usize, u8) -> (u8, bool)
{
    //todo - this entire algorithm is very inefficient in terms of memory usage...


    let mut remaining_bits_in_message = message_length_in_bits;

    for i in 0..original.len() {
        output_map[i] = original[i];
    }

    let mut all_indices: Vec<usize> = (0..original.len()).collect();
    let mut rng = rand::thread_rng();

    while remaining_bits_in_message > 0 {
        all_indices.shuffle(&mut rng);
        let mut success_counter = 0;

        let mut step_from = 0; //use up all indices before
        while remaining_bits_in_message > 0 {
            if step_from >= all_indices.len() { //step from is changed at the bottom of the while
                if success_counter == 0 {
                    return Err(DifCodeError::InternalCapacityReached(message_length_in_bits - remaining_bits_in_message));
                } else {
                    break;
                }
            }

            let step_to = (step_from + remaining_bits_in_message).min(all_indices.len());
            let selected_indices = &all_indices[step_from..step_to];

            for sel_i in selected_indices {
                let original_value = original[*sel_i];
                let allowed_change = output_map[*sel_i];
                let old_difference = distance(original_value, allowed_change);
                let (max_difference, initial_direction_positive) = change_constraint_calculator(*sel_i, original_value);
                //with max differences != to the numbits boundaries this can

                if old_difference < max_difference {
                    let num_bits_currently_encodable = get_max_num_bits_encodable(old_difference);
                    if let Some(mut new_difference) = calculate_worst_case_difference_for(num_bits_currently_encodable + 1) {
                        if new_difference <= max_difference {
                            if original_value == allowed_change {
                                let is_direction_positive = initial_direction_positive;
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
            }

            remaining_bits_in_message -= success_counter;
            step_from = step_to;
        }
    }

    Ok(())
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














pub fn create_minimal_evenly_random_max_area_average_allowed_changes_map(message: &[u8], original: &DifCodeImage) -> DifCodeResult<Vec<u8>> {
    let mut output_map = vec![0; original.len()];
    write_minimal_evenly_random_max_area_average_allowed_changes_map_for(message, original, &mut output_map)?;
    Ok(output_map)
}
pub fn write_minimal_evenly_random_max_area_average_allowed_changes_map_for(message: &[u8], original: &DifCodeImage, output_map: &mut dyn EncodingContainer) -> DifCodeResult<()> {
    write_minimal_evenly_random_max_area_average_allowed_changes_map(get_length_in_bits(message), original, output_map)
}
pub fn write_minimal_evenly_random_max_area_average_allowed_changes_map(message_length_in_bits: usize, original: &DifCodeImage, output_map: &mut dyn EncodingContainer) -> DifCodeResult<()> {
    let integral_image = original.generate_integral_image_for_rgb();
    let change_constraint_calculator = move |index, ov| {
        let (x, y, z) = DifCodeImage::index_to_xyz_with_wh(index, integral_image.width(), integral_image.height());
        let average = integral_image.average_in_radius(x, y, z, 10);
        let max_difference = distance(ov, average);
        let initial_direction_positive = ov < average; //if ov is smaller than average, then we make a positive change
        (max_difference, initial_direction_positive)
    };

    write_minimal_evenly_random_allowed_changes_map(message_length_in_bits, original, change_constraint_calculator, output_map)
}